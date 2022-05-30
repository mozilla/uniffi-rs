/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;

use anyhow::{anyhow, bail, Context};
use camino::Utf8Path;
use fs_err as fs;
use serde::de::DeserializeOwned;
use uniffi_meta::FnMetadata;

use crate::interface::ComponentInterface;

fn parse_json_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    // Buffer in String because parsing using io::Read is slow:
    // https://github.com/serde-rs/json/issues/160
    let s = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

pub(crate) fn add_macro_metadata(
    iface: &mut ComponentInterface,
    crate_root: &Utf8Path,
) -> anyhow::Result<()> {
    let metadata_dir = &crate_root.join(".uniffi").join("metadata");

    if !metadata_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(metadata_dir)? {
        let entry = entry?;
        let file_name = &entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("non-utf8 file names are not supported"))?;

        let file_basename = file_name.strip_suffix(".json").ok_or_else(|| {
            anyhow!(
                "expected only JSON files in `{}`, found `{}`",
                metadata_dir,
                file_name
            )
        })?;

        let mut segments = match file_basename.strip_prefix("mod.") {
            Some(rest) => rest.split('.'),
            None => bail!("expected filename to being with `mod.`"),
        };

        let _mod_path = segments
            .next()
            .context("incomplete filename")?
            .replace('$', "::");

        match segments.next() {
            Some("fn") => {
                let meta: FnMetadata = parse_json_file(entry.path())?;
                iface.add_function_definition(meta.into())?;
            }
            Some("impl") => {
                return Err(anyhow!("impl blocks are not yet supported")
                    .context(format!("processing {}", entry.path().display())));
            }
            Some("type") => {
                return Err(anyhow!("custom types are not yet supported")
                    .context(format!("processing {}", entry.path().display())))
            }
            _ => {
                bail!(
                    "unexpected filename, expected pattern of `mod.<mod_name>.fn.<fn_name>.json`"
                );
            }
        }
    }

    iface.check_consistency()?;
    iface.derive_ffi_funcs()?;

    Ok(())
}
