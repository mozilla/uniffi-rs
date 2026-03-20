use std::{env, fs};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::MetadataCommand;
use clap::Parser;
use uniffi_bindgen::bindings::{
    kotlin_test::run_script as kotlin_run_script, python_test::run_script as python_run_script,
    swift_test::run_script as swift_run_script, RunScriptOptions,
};

use uniffi_benchmarks::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let script_args: Vec<String> = std::iter::once(String::from("--"))
        .chain(env::args())
        .collect();

    let options = RunScriptOptions {
        show_compiler_messages: args.compiler_messages,
    };

    let workspace_root = MetadataCommand::new().exec()?.workspace_root;
    let bindings_dir = workspace_root.join("benchmarks/bindings");

    if args.clean {
        let sample_dir = workspace_root.join("target/uniffi-bench/samples/");
        println!("Deleting {sample_dir}");
        std::fs::remove_dir_all(sample_dir)?;
        return Ok(());
    }

    if args.should_run_python() {
        python_run_script(
            script_tempdir(&workspace_root, "python")?.as_str(),
            "uniffi-benchmarks",
            bindings_dir.join("run_benchmarks.py").as_str(),
            script_args.clone(),
            &options,
        )?;
    }

    if args.should_run_kotlin() {
        kotlin_run_script(
            script_tempdir(&workspace_root, "kotlin")?.as_str(),
            "uniffi-benchmarks",
            bindings_dir.join("run_benchmarks.kts").as_str(),
            script_args.clone(),
            &options,
        )?;
    }

    if args.should_run_swift() {
        swift_run_script(
            script_tempdir(&workspace_root, "swift")?.as_str(),
            "uniffi-benchmarks",
            bindings_dir.join("run_benchmarks.swift").as_str(),
            script_args,
            &options,
        )?;
    }

    Ok(())
}

fn script_tempdir(workspace_root: &Utf8Path, language_name: &str) -> Result<Utf8PathBuf> {
    let tempdir = workspace_root
        .join("target/uniffi-bench/tmp/")
        .join(language_name);
    if !tempdir.exists() {
        fs::create_dir_all(&tempdir).with_context(|| format!("while creating {tempdir}"))?;
    }
    Ok(tempdir)
}
