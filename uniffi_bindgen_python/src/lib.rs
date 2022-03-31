/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use uniffi_bindgen::{generate_external_bindings, BindingGenerator, ComponentInterface};

mod gen_python;

struct PythonBindingGenerator {
    stdout: bool,
    try_format_code: bool,
}

impl PythonBindingGenerator {
    fn new(stdout: bool, try_format_code: bool) -> Self {
        Self {
            stdout,
            try_format_code,
        }
    }

    fn create_writer(
        &self,
        ci: &ComponentInterface,
        out_dir: &Path,
    ) -> anyhow::Result<Box<dyn Write>> {
        if self.stdout {
            Ok(Box::new(std::io::stdout()))
        } else {
            let filename = format!("{}.py", ci.namespace());
            let out_path = out_dir.join(&filename);
            Ok(Box::new(
                File::create(&out_path).context(format!("Failed to create {:?}", filename))?,
            ))
        }
    }
}

impl BindingGenerator for PythonBindingGenerator {
    type Config = gen_python::Config;

    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        let mut writer = self.create_writer(&ci, out_dir)?;
        let mut bindings = gen_python::generate_python_bindings(&config, &ci)?;

        if self.try_format_code {
            match Command::new("yaph")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    child
                        .stdin
                        .take()
                        .expect("Failed to open stdin")
                        .write_all(bindings.as_bytes())?;
                    let output = child.wait_with_output().expect("Failed to read stdout");
                    bindings = String::from_utf8(output.stdout).expect("Error decoded yaph output");
                }
                Err(e) => println!("Warning: Unable to auto-format Python using yaph: {:?}", e),
            }
        }
        write!(writer, "{}", bindings)?;
        Ok(())
    }
}

pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = clap::App::new("uniffi-bindgen-python")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("stdout")
                .long("--stdout")
                .help("Write output to STDOUT"),
        )
        .arg(
            clap::Arg::with_name("no_format")
            .long("--no-format")
            .help("Do not format the generated code with rustfmt (useful for maintainers)"),
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

    let binding_generator = PythonBindingGenerator::new(
        matches.is_present("stdout"),
        !matches.is_present("no_format"),
    );
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
