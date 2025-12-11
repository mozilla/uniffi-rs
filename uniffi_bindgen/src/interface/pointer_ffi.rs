/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI definitions for the pointer FFI
//!
//! These exist in parallel with the normal/legacy FFI.  If the user enables the `pointer-ffi`
//! feature, then both versions of the FFI will be generated.  The pointer FFI symbols are prefixed
//! with `uniffi_ptr_` to avoid any conflicts.

use super::{ComponentInterface, UniffiTrait};

impl ComponentInterface {
    pub fn pointer_ffi_function_names(&self) -> impl Iterator<Item = String> + '_ {
        [].into_iter()
            // Functions
            .chain(
                self.function_definitions()
                    .iter()
                    .map(|f| f.ffi_func().pointer_ffi_name()),
            )
            // Constructors
            .chain(self.object_definitions().iter().flat_map(|o| {
                o.constructors()
                    .into_iter()
                    .map(|c| c.ffi_func().pointer_ffi_name())
            }))
            // Methods
            .chain(
                self.enum_definitions()
                    .iter()
                    .flat_map(|e| e.methods())
                    .chain(self.record_definitions().iter().flat_map(|r| r.methods()))
                    .chain(self.object_definitions().iter().flat_map(|o| o.methods()))
                    .map(|m| m.ffi_func().pointer_ffi_name()),
            )
            // UniFFI trait methods
            .chain(
                self.enum_definitions()
                    .iter()
                    .flat_map(|e| e.uniffi_traits())
                    .chain(
                        self.record_definitions()
                            .iter()
                            .flat_map(|r| r.uniffi_traits()),
                    )
                    .chain(
                        self.object_definitions()
                            .iter()
                            .flat_map(|o| o.uniffi_traits()),
                    )
                    .flat_map(|ut| match ut {
                        UniffiTrait::Display { fmt: m }
                        | UniffiTrait::Debug { fmt: m }
                        | UniffiTrait::Hash { hash: m }
                        | UniffiTrait::Ord { cmp: m } => vec![m],
                        UniffiTrait::Eq { eq, ne } => vec![eq, ne],
                    })
                    .map(|m| m.ffi_func().pointer_ffi_name()),
            )
    }
}
