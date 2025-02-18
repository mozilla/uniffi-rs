/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add info about FFI functions to manage object handles

use uniffi_meta;

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let crate_name = module.crate_name.clone();
    module.visit_mut(|int: &mut Interface| {
        int.ffi_func_clone =
            RustFfiFunctionName(uniffi_meta::clone_fn_symbol_name(&crate_name, &int.name));
        int.ffi_func_free =
            RustFfiFunctionName(uniffi_meta::free_fn_symbol_name(&crate_name, &int.name));
    });
    Ok(())
}
