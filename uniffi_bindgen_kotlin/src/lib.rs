/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use uniffi_bindgen::{
    interface::ComponentInterface, BindingGenerator, MergeWith, BindingGeneratorConfig,
};
pub mod gen_kotlin;
pub use gen_kotlin::{generate_bindings, Config};
use serde::Deserialize;


// TODO: Extract this from cargo metadata
const UNIFFI_VERSION: &str = "0.16.0";
use anyhow::{bail, Context, Result};
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command
};


pub fn run_main() -> anyhow::Result<()> {
    let matches = clap::App::new("uniffi-bindgen")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("uniffi_version").required(true)
            .help("The uniffi version uniffi_bindgen_kotlin is going to run with. NOTE: The version specified here, must be the same one, uniffi_bingen_kotlin uses internally")
        )
        .subcommand(
            clap::SubCommand::with_name("generate")
                .about("Generate the foreign language bindings")
                .arg(
                    clap::Arg::with_name("out_dir")
                        .long("--out-dir")
                        .short("-o")
                        .takes_value(true)
                        .help("Directory in which to write generated files. Default is same folder as .udl file."),
                )
                .arg(
                    clap::Arg::with_name("no_format")
                        .long("--no-format")
                        .help("Do not try to format the generated bindings"),
                )
                .arg(clap::Arg::with_name("udl_file").required(true))
                .arg(
                    clap::Arg::with_name("config")
                    .long("--config-path")
                    .takes_value(true)
                    .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("test")
            .about("Run test scripts against foreign language bindings")
            .arg(clap::Arg::with_name("cdylib_dir").required(true).help("Path to the directory containing the cdylib the scripts will be testing against."))
            .arg(clap::Arg::with_name("udl_file").required(true))
            .arg(clap::Arg::with_name("test_scripts").required(true).multiple(true).help("Foreign language(s) test scripts to run"))
            .arg(
                clap::Arg::with_name("config")
                .long("--config-path")
                .takes_value(true)
                .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess from the UDL's file location.")
            )
        )
        .get_matches();
    let uniffi_version = matches
        .value_of_os("uniffi_version")
        .expect("`uniffi_version is a required argument"); // required!
    if uniffi_version != UNIFFI_VERSION {
        bail!("Invalid uniffi_version, this version of uniffi_bingen_kotlin only works with version {}, found {:?}", UNIFFI_VERSION, uniffi_version)
    }
    match matches.subcommand() {
        ("generate", Some(m)) => {
            uniffi_bindgen::generate_backend_bindings::<KotlinBackend, &OsStr>(
                m.value_of_os("udl_file").unwrap(), // Required
                m.value_of_os("config"),
                m.value_of_os("out_dir"),
                !m.is_present("no_format"),
            )
        }
        //TODO: Add support for a test subcommand
        // that does the same dance as the "generate"
        _ => bail!("No command specified; try `--help` for some help"),
    }
}

pub struct KotlinBackend {
    config: UniffiConfig
}

#[derive(Debug, Default, Deserialize)]
pub struct UniffiConfig {
    #[serde(default)]
    kotlin: Config
}

impl MergeWith for UniffiConfig {
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            kotlin: self.kotlin.merge_with(&other.kotlin)
        }
    }
}

impl From<&ComponentInterface> for UniffiConfig {
    fn from(ci: &ComponentInterface) -> Self {
        Self {
            kotlin: ci.into()
        }
    }
}

impl BindingGeneratorConfig for UniffiConfig {}


impl BindingGenerator for KotlinBackend {
    type Config = UniffiConfig;
    fn new(config: UniffiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config
        })
    }

    fn write_bindings<P: AsRef<Path>>(
        &self,
        ci: &uniffi_bindgen::interface::ComponentInterface,
        out_dir: P,
    ) -> anyhow::Result<()> {
        let mut kt_file = full_bindings_path(&self.config.kotlin, out_dir.as_ref())?;
        std::fs::create_dir_all(&kt_file)?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        let mut f = File::create(&kt_file).context("Failed to create .kt file for bindings")?;
        write!(f, "{}", generate_bindings(&self.config.kotlin, ci)?)?;

        Ok(())
    }
    /// Generate kotlin bindings for the given namespace, then use the kotlin
    /// command-line tools to compile them into a .jar file.
    fn compile_bindings<P: AsRef<Path>>(
        &self,
        ci: &ComponentInterface,
        out_dir: P,
    ) -> anyhow::Result<()> {
        let mut kt_file = full_bindings_path(&self.config.kotlin, out_dir.as_ref())?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        let mut jar_file = PathBuf::from(out_dir.as_ref());
        jar_file.push(format!("{}.jar", ci.namespace()));
        let status = Command::new("kotlinc")
            // Our generated bindings should not produce any warnings; fail tests if they do.
            .arg("-Werror")
            // Reflect $CLASSPATH from the environment, to help find `jna.jar`.
            .arg("-classpath")
            .arg(env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
            .arg(&kt_file)
            .arg("-d")
            .arg(jar_file)
            .spawn()
            .context("Failed to spawn `kotlinc` to compile the bindings")?
            .wait()
            .context("Failed to wait for `kotlinc` when compiling the bindings")?;
        if !status.success() {
            bail!("running `kotlinc` failed")
        }
        Ok(())
    }

    /// Execute the specifed kotlin script, with classpath based on the generated
    /// artifacts in the given output directory.
    fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> Result<()> {
        let mut classpath = env::var_os("CLASSPATH").unwrap_or_else(|| OsString::from(""));
        // This lets java find the compiled library for the rust component.
        classpath.push(":");
        classpath.push(out_dir.as_ref());
        // This lets java use any generate .jar files containing bindings for the rust component.
        for entry in PathBuf::from(out_dir.as_ref())
            .read_dir()
            .context("Failed to list target directory when running Kotlin script")?
        {
            let entry = entry.context("Directory listing failed while running Kotlin script")?;
            if let Some(ext) = entry.path().extension() {
                if ext == "jar" {
                    classpath.push(":");
                    classpath.push(entry.path());
                }
            }
        }
        let mut cmd = Command::new("kotlinc");
        // Make sure it can load the .jar and its dependencies.
        cmd.arg("-classpath").arg(classpath);
        // Code that wants to use an API with unsigned types, must opt in to this experimental Kotlin feature.
        // Specify it here in order to not have to worry about that when writing tests.
        cmd.arg("-Xopt-in=kotlin.ExperimentalUnsignedTypes");
        // Enable runtime assertions, for easy testing etc.
        cmd.arg("-J-ea");
        // Our test scripts should not produce any warnings.
        cmd.arg("-Werror");
        cmd.arg("-script").arg(script_file.as_ref());
        let status = cmd
            .spawn()
            .context("Failed to spawn `kotlinc` to run Kotlin script")?
            .wait()
            .context("Failed to wait for `kotlinc` when running Kotlin script")?;
        if !status.success() {
            bail!("running `kotlinc` failed")
        }
        Ok(())
    }
}

fn full_bindings_path(config: &Config, out_dir: &Path) -> Result<PathBuf> {
    let package_path: PathBuf = config.package_name().split('.').collect();
    Ok(PathBuf::from(out_dir).join(package_path))
}

/// Generate kotlin bindings for the given namespace, then use the kotlin
/// command-line tools to compile them into a .jar file.
pub fn compile_bindings(config: &Config, ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut kt_file = full_bindings_path(config, out_dir)?;
    kt_file.push(format!("{}.kt", ci.namespace()));
    let mut jar_file = PathBuf::from(out_dir);
    jar_file.push(format!("{}.jar", ci.namespace()));
    let status = Command::new("kotlinc")
        // Our generated bindings should not produce any warnings; fail tests if they do.
        .arg("-Werror")
        // Reflect $CLASSPATH from the environment, to help find `jna.jar`.
        .arg("-classpath")
        .arg(env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
        .arg(&kt_file)
        .arg("-d")
        .arg(jar_file)
        .spawn()
        .context("Failed to spawn `kotlinc` to compile the bindings")?
        .wait()
        .context("Failed to wait for `kotlinc` when compiling the bindings")?;
    if !status.success() {
        bail!("running `kotlinc` failed")
    }
    Ok(())
}