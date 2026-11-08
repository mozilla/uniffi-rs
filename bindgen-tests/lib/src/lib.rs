/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(feature = "udl")]
uniffi::include_scaffolding!("uniffi_bindgen_tests");

#[cfg(not(feature = "udl"))]
uniffi::setup_scaffolding!("uniffi_bindgen_tests");

#[cfg(feature = "bytes")]
pub mod bytes;

#[cfg(feature = "callback_interfaces")]
pub mod callback_interfaces;

#[cfg(feature = "collections")]
pub mod collections;

#[cfg(feature = "custom_types")]
pub mod custom_types;

#[cfg(feature = "defaults")]
pub mod defaults;

#[cfg(feature = "enums")]
pub mod enums;

#[cfg(feature = "errors")]
pub mod errors;

#[cfg(feature = "external-types")]
pub mod external_types;

#[cfg(feature = "futures")]
pub mod futures;

#[cfg(feature = "interfaces")]
pub mod interfaces;

#[cfg(feature = "options")]
pub mod options;

#[cfg(feature = "primitive_types")]
pub mod primitive_types;

#[cfg(feature = "records")]
pub mod records;

#[cfg(feature = "recursive_types")]
pub mod recursive_types;

#[cfg(feature = "references")]
pub mod references;

#[cfg(feature = "renames")]
pub mod renames;

#[cfg(feature = "rust_traits")]
pub mod rust_traits;

#[cfg(feature = "simple_fns")]
pub mod simple_fns;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "trait_interfaces")]
pub mod trait_interfaces;

#[cfg(feature = "udl")]
pub mod udl;

// UDL items must be re-exported by the top-level crate
#[cfg(feature = "udl")]
pub use udl::*;
