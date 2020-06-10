/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    uniffi::generate_component_scaffolding("arithmetic.idl");
    uniffi::generate_kotlin_bindings("arithmetic.idl");
    compile_kotlin_example();
}

fn compile_kotlin_example() {
    println!("cargo:rerun-if-changed=main.kt");
    let mut gen_file = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    gen_file.push("arithmetic.kt");
    // There's a whole lot of packaging and dependency-management stuff to figure out here.
    // For now I just want to hack it into compiling and running some generated kotlin code.
    let status = std::process::Command::new("kotlinc")
        .arg("-include-runtime")
        .arg("-classpath").arg("/Users/rfk/.gradle/caches/modules-2/files-2.1/net.java.dev.jna/jna/5.2.0/ed8b772eb077a9cb50e44e90899c66a9a6c00e67/jna-5.2.0.jar")
        .arg("../../src/Helpers.kt")
        .arg(gen_file)
        .arg("main.kt")
        .arg("-d").arg("arithmetic.jar")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(status.success());
}