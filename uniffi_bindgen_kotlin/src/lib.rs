/* This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uniffi_bindgen::{generate_external_bindings, BindingGenerator, ComponentInterface};

mod gen_kotlin;

struct KotlinBindingGenerator {
    stdout: bool,
}

impl KotlinBindingGenerator {
    fn new(stdout: bool) -> Self {
        Self { stdout }
    }

    fn create_writer(
        &self,
        ci: &ComponentInterface,
        out_dir: &Path,
    ) -> anyhow::Result<Box<dyn Write>> {
        if self.stdout {
            Ok(Box::new(std::io::stdout()))
        } else {
            let filename = format!("{}.kt", ci.namespace());
            let out_path = out_dir.join(&filename);
            Ok(Box::new(
                File::create(&out_path).context(format!("Failed to create {:?}", filename))?,
            ))
        }
    }
}

impl BindingGenerator for KotlinBindingGenerator {
    type Config = gen_kotlin::Config;

    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        let mut writer = self.create_writer(&ci, out_dir)?;
        write!(writer, "{}", gen_kotlin::generate_bindings(&config, &ci)?)?;
        Ok(())
    }
}

pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = clap::App::new("uniffi-bindgen-kotlin")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("stdout")
                .long("--stdout")
                .help("Write output to STDOUT"),
        )
        .arg(
            clap::Arg::with_name("out_dir")
                .long("--out-dir")
                .short("-o")
                .takes_value(true)
                .help("Directory in which to write generated files. Default is same folder as .udl file."),
        )
        .arg(clap::Arg::with_name("udl_file").required(true))
        .arg(
            clap::Arg::with_name("config")
            .long("--config-path")
            .takes_value(true)
            .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
        )
        .get_matches_from(args);

    let binding_generator = KotlinBindingGenerator::new(matches.is_present("stdout"));
    generate_external_bindings(
        binding_generator,
        matches.value_of_os("udl_file").unwrap(), // Required
        matches.value_of_os("config"),
        matches.value_of_os("out_dir"),
    )
}

pub fn run_main() -> Result<()> {
    run(std::env::args_os())
}
