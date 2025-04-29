/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Reexport items from other uniffi creates
pub use uniffi_core::*;
pub use uniffi_macros::*;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "bindgen-tests")]
pub use uniffi_bindgen::bindings::{kotlin_test, python_test, ruby_test, swift_test};

#[cfg(all(feature = "cargo-metadata", feature = "bindgen"))]
pub use uniffi_bindgen::cargo_metadata::CrateConfigSupplier as CargoMetadataConfigSupplier;
#[cfg(feature = "bindgen")]
pub use uniffi_bindgen::library_mode::generate_bindings as generate_bindings_library_mode;
#[cfg(feature = "bindgen")]
pub use uniffi_bindgen::{
    bindings::{KotlinBindingGenerator, RubyBindingGenerator, SwiftBindingGenerator},
    generate_bindings, print_repr,
};
#[cfg(feature = "build")]
pub use uniffi_build::{generate_scaffolding, generate_scaffolding_for_crate};
#[cfg(feature = "bindgen-tests")]
pub use uniffi_macros::build_foreign_language_testcases;

#[cfg(feature = "cli")]
pub use cli::*;

#[cfg(test)]
mod test {
    #[test]
    fn trybuild_ui_tests() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }
}
