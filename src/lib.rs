/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::prelude::*;
use std::{
    env,
    collections::HashMap,
    convert::TryFrom, convert::TryInto,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::bail;
use anyhow::Result;

pub mod interface;
pub mod scaffolding;
pub mod kotlin;
pub mod support;

use scaffolding::RustScaffolding;
use kotlin::KotlinWrapper;

fn slurp_file(file_name: &str) -> Result<String> {
    let mut contents = String::new();
    let mut f = File::open(file_name)?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

// Call this when building the rust crate that implements the specified interface.
// It will generate a bunch of the infrastructural rust code for implementing
// the interface, such as the `extern "C"` function definitions and record data types.
//
pub fn generate_component_scaffolding(idl_file: &str) {
    println!("cargo:rerun-if-changed={}", idl_file);
    let idl = slurp_file(idl_file).unwrap();
    let component= idl.parse::<interface::ComponentInterface>().unwrap();
    let mut filename = Path::new(idl_file).file_stem().unwrap().to_os_string();
    filename.push(".uniffi.rs");
    let mut out_file = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_file.push(filename);
    let mut f = File::create(out_file).unwrap();
    write!(f, "{}", RustScaffolding::new(&component)).unwrap();
}


// Call this to generate Kotlin bindings to load and call into the specified interface.
// XXX TODO: need to have more params here to control how and where it's generated.
// Writing into the OUT_DIR is probably not the write thing for foreign language bindings.
pub fn generate_kotlin_bindings(idl_file: &str) {
    let idl = slurp_file(idl_file).unwrap();
    let component= idl.parse::<interface::ComponentInterface>().unwrap();
    let mut filename = Path::new(idl_file).file_stem().unwrap().to_os_string();
    filename.push(".kt");
    let mut out_file = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_file.push(filename);
    let mut f = File::create(out_file).unwrap();
    write!(f, "{}", KotlinWrapper::new(kotlin::Config{package_name: "uniffi_test".to_string()}, &component)).unwrap();
}


// Call this to generate Swift bindings to load and call into the specified interface.
// XXX TODO: actually, you know, implement it...
pub fn generate_swift_bindings(idl_file: &str) {
    panic!("haven't implemented generation of swift bindings yet");
}


// Call this to generate XPCOM JS bindings to load and call into the specified interface.
// XXX TODO: actually, you know, implement it...
pub fn generate_xpcom_js_bindings(idl_file: &str) {
    panic!("haven't implemented generation of xpcom js bindings yet");
}

// Call this to generate Rust bindings to load and call into the specified interface.
// These bindings are what *external* consumers of the component should call, in order
// to access it through the FFI. For example, we might use it to generate bindings for
// glean so we can use glean from outside the megazord.
// XXX TODO: actually, you know, implement it...
pub fn generate_rust_bindings(idl_file: &str) {
    panic!("haven't implemented generation of rust bindings yet");
}