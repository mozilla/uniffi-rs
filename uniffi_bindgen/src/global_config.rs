/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, fs};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use crate::{merge_toml, BindgenPaths, BindgenPathsLayer};

/// A [BindgenPathsLayer] backed by an explicit `[crate-roots]` mapping.
pub struct CrateRootsLayer {
    roots: HashMap<String, Utf8PathBuf>,
}

impl BindgenPathsLayer for CrateRootsLayer {
    fn get_crate_root(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.roots.get(crate_name).cloned()
    }
}

/// Deserialized representation of a global config TOML file.
#[derive(Deserialize, Default)]
struct GlobalConfigFile {
    #[serde(rename = "crate-roots", default)]
    crate_roots: HashMap<String, String>,
    #[serde(default)]
    defaults: toml::value::Table,
    #[serde(default)]
    crates: HashMap<String, toml::value::Table>,
}

/// Global configuration for UniFFI.
///
/// Holds defaults and per-crate overrides from a global config file.
/// Config resolution requires a `&BindgenPaths` to locate per-crate `uniffi.toml` files.
#[derive(Default)]
pub struct GlobalConfig {
    defaults: toml::value::Table,
    crate_overrides: HashMap<String, toml::value::Table>,
}

impl GlobalConfig {
    /// Parse a global config file.
    ///
    /// Returns the `GlobalConfig` and an optional `CrateRootsLayer`. If a
    /// `CrateRootsLayer` is returned, add it to your `BindgenPaths` before
    /// calling `get_config`.
    pub fn from_file(path: &Utf8Path) -> Result<(Self, Option<CrateRootsLayer>)> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read file: {:?}", path))?;

        // Check for old-style flat config files before full parse
        let raw: toml::value::Table =
            toml::de::from_str(&contents).with_context(|| format!("parse toml: {:?}", path))?;

        let has_global_config_keys = raw.contains_key("crate-roots")
            || raw.contains_key("defaults")
            || raw.contains_key("crates");

        if !has_global_config_keys && !raw.is_empty() {
            eprintln!(
                "warning: {path} looks like an old-style --config override file. \
                The --config flag now expects a global config file with [defaults], \
                [crates.<name>], and/or [crate-roots] sections. \
                Old-style flat config files are no longer supported."
            );
            return Ok((Self::default(), None));
        }

        let file: GlobalConfigFile =
            toml::de::from_str(&contents).with_context(|| format!("parse toml: {:?}", path))?;

        let crate_roots_layer = if file.crate_roots.is_empty() {
            None
        } else {
            let base_dir = path.parent().unwrap_or(Utf8Path::new("."));
            let roots = file
                .crate_roots
                .into_iter()
                .map(|(name, rel_path)| (name, base_dir.join(rel_path)))
                .collect();
            Some(CrateRootsLayer { roots })
        };

        Ok((
            Self {
                defaults: file.defaults,
                crate_overrides: file.crates,
            },
            crate_roots_layer,
        ))
    }

    /// Get the merged config table for a crate.
    /// Recursively and structurally applies TOML, later wins:
    /// `[defaults]` from the global config; then the crate's `uniffi.toml` via `BindgenPaths;
    /// then `[crates.{name}]` from the global config.
    pub fn get_config(&self, paths: &BindgenPaths, crate_name: &str) -> Result<toml::value::Table> {
        let mut config = self.defaults.clone();

        if let Some(config_path) = paths.get_config_path(crate_name) {
            if config_path.exists() {
                let contents = fs::read_to_string(&config_path)
                    .with_context(|| format!("read file: {:?}", config_path))?;
                let crate_config: toml::value::Table = toml::de::from_str(&contents)
                    .with_context(|| format!("parse toml: {:?}", config_path))?;
                merge_toml(&mut config, crate_config);
            }
        }

        if let Some(overrides) = self.crate_overrides.get(crate_name) {
            merge_toml(&mut config, overrides.clone());
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use tempfile::TempDir;

    // A simple BindgenPathsLayer that maps one crate to a fixed root path.
    struct StaticLayer {
        crate_name: String,
        crate_root: Utf8PathBuf,
    }

    impl BindgenPathsLayer for StaticLayer {
        fn get_crate_root(&self, crate_name: &str) -> Option<Utf8PathBuf> {
            if crate_name == self.crate_name {
                Some(self.crate_root.clone())
            } else {
                None
            }
        }
    }

    fn write_file(dir: &TempDir, name: &str, contents: &str) -> Utf8PathBuf {
        let path = Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    fn paths_for(crate_name: &str, root: &Utf8PathBuf) -> BindgenPaths {
        let mut paths = BindgenPaths::default();
        paths.add_layer(StaticLayer {
            crate_name: crate_name.to_string(),
            crate_root: root.clone(),
        });
        paths
    }

    #[test]
    fn test_empty_global_config() {
        let dir = TempDir::new().unwrap();
        let paths = paths_for(
            "my_crate",
            &Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap(),
        );
        let config = GlobalConfig::default();
        let table = config.get_config(&paths, "my_crate").unwrap();
        assert!(table.is_empty());
    }

    #[test]
    fn test_defaults_only() {
        let dir = TempDir::new().unwrap();
        let global_path = write_file(
            &dir,
            "global.toml",
            r#"
                [defaults.bindings.swift]
                ffi_module_name = "MyFFI"
            "#,
        );
        let (config, roots_layer) = GlobalConfig::from_file(&global_path).unwrap();
        assert!(roots_layer.is_none());

        // Point to a crate root with no uniffi.toml
        let crate_root = Utf8PathBuf::from_path_buf(dir.path().join("my_crate")).unwrap();
        fs::create_dir_all(&crate_root).unwrap();
        let paths = paths_for("my_crate", &crate_root);

        let table = config.get_config(&paths, "my_crate").unwrap();
        assert_eq!(
            table["bindings"]["swift"]["ffi_module_name"]
                .as_str()
                .unwrap(),
            "MyFFI"
        );
    }

    #[test]
    fn test_crate_config_overrides_defaults() {
        let dir = TempDir::new().unwrap();
        let global_path = write_file(
            &dir,
            "global.toml",
            r#"
                [defaults.bindings.swift]
                ffi_module_name = "DefaultFFI"
                generate_immutable_records = true
            "#,
        );
        let crate_root = Utf8PathBuf::from_path_buf(dir.path().join("my_crate")).unwrap();
        fs::create_dir_all(&crate_root).unwrap();
        // crate's uniffi.toml overrides ffi_module_name but not generate_immutable_records
        fs::write(
            crate_root.join("uniffi.toml"),
            r#"
                [bindings.swift]
                ffi_module_name = "CrateFFI"
            "#,
        )
        .unwrap();

        let (config, _) = GlobalConfig::from_file(&global_path).unwrap();
        let paths = paths_for("my_crate", &crate_root);
        let table = config.get_config(&paths, "my_crate").unwrap();

        // crate wins on the overlapping key
        assert_eq!(
            table["bindings"]["swift"]["ffi_module_name"]
                .as_str()
                .unwrap(),
            "CrateFFI"
        );
        // default fills in the rest
        assert!(table["bindings"]["swift"]["generate_immutable_records"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_per_crate_overrides_win() {
        let dir = TempDir::new().unwrap();
        let global_path = write_file(
            &dir,
            "global.toml",
            r#"
                [defaults.bindings.swift]
                ffi_module_name = "DefaultFFI"

                [crates.my_crate.bindings.swift]
                ffi_module_name = "OverrideFFI"
                ffi_module_filename = "my_crate_ffi"
            "#,
        );
        let crate_root = Utf8PathBuf::from_path_buf(dir.path().join("my_crate")).unwrap();
        fs::create_dir_all(&crate_root).unwrap();
        fs::write(
            crate_root.join("uniffi.toml"),
            r#"
                [bindings.swift]
                ffi_module_name = "CrateFFI"
            "#,
        )
        .unwrap();

        let (config, _) = GlobalConfig::from_file(&global_path).unwrap();
        let paths = paths_for("my_crate", &crate_root);
        let table = config.get_config(&paths, "my_crate").unwrap();

        // per-crate override wins over both default and crate uniffi.toml
        assert_eq!(
            table["bindings"]["swift"]["ffi_module_name"]
                .as_str()
                .unwrap(),
            "OverrideFFI"
        );
        // per-crate override also adds new keys
        assert_eq!(
            table["bindings"]["swift"]["ffi_module_filename"]
                .as_str()
                .unwrap(),
            "my_crate_ffi"
        );
    }

    #[test]
    fn test_deep_merge() {
        let dir = TempDir::new().unwrap();
        let global_path = write_file(
            &dir,
            "global.toml",
            r#"
                [defaults.bindings.swift]
                ffi_module_name = "SharedFFI"

                [crates.my_crate.bindings.kotlin]
                package_name = "com.example"
            "#,
        );
        let crate_root = Utf8PathBuf::from_path_buf(dir.path().join("my_crate")).unwrap();
        fs::create_dir_all(&crate_root).unwrap();
        fs::write(
            crate_root.join("uniffi.toml"),
            r#"
                [bindings.swift]
                ffi_module_filename = "my_crate_ffi"
            "#,
        )
        .unwrap();

        let (config, _) = GlobalConfig::from_file(&global_path).unwrap();
        let paths = paths_for("my_crate", &crate_root);
        let table = config.get_config(&paths, "my_crate").unwrap();
        let bindings = &table["bindings"];

        // swift keys from both default and crate uniffi.toml are present
        assert_eq!(
            bindings["swift"]["ffi_module_name"].as_str().unwrap(),
            "SharedFFI"
        );
        assert_eq!(
            bindings["swift"]["ffi_module_filename"].as_str().unwrap(),
            "my_crate_ffi"
        );
        // kotlin key from per-crate override is present
        assert_eq!(
            bindings["kotlin"]["package_name"].as_str().unwrap(),
            "com.example"
        );
    }

    #[test]
    fn test_crate_roots_layer() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("crates").join("my_crate");
        fs::create_dir_all(&sub).unwrap();

        let global_path = write_file(
            &dir,
            "global.toml",
            r#"
                [crate-roots]
                my_crate = "crates/my_crate"
            "#,
        );

        let (config, roots_layer) = GlobalConfig::from_file(&global_path).unwrap();
        let roots_layer = roots_layer.expect("expected CrateRootsLayer");

        let mut paths = BindgenPaths::default();
        paths.add_layer(roots_layer);

        // The crate root should resolve to an absolute path under dir
        let root = paths.get_crate_root("my_crate").unwrap();
        assert!(root.is_absolute());
        assert!(root.ends_with("crates/my_crate"));

        // Config path should be {root}/uniffi.toml
        let config_path = paths.get_config_path("my_crate").unwrap();
        assert!(config_path.ends_with("uniffi.toml"));

        // get_config works even with no uniffi.toml present
        let table = config.get_config(&paths, "my_crate").unwrap();
        assert!(table.is_empty());
    }

    #[test]
    fn test_old_style_config_returns_default() {
        let dir = TempDir::new().unwrap();
        // Old-style: flat keys not under [defaults], [crates.*], or [crate-roots]
        let global_path = write_file(
            &dir,
            "old.toml",
            r#"
                [bindings.swift]
                ffi_module_name = "OldFFI"
            "#,
        );

        let (config, roots_layer) = GlobalConfig::from_file(&global_path).unwrap();
        assert!(roots_layer.is_none());

        let crate_root = Utf8PathBuf::from_path_buf(dir.path().join("my_crate")).unwrap();
        fs::create_dir_all(&crate_root).unwrap();
        let paths = paths_for("my_crate", &crate_root);

        // Old-style config is ignored; result is empty
        let table = config.get_config(&paths, "my_crate").unwrap();
        assert!(table.is_empty());
    }
}
