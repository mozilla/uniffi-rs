/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod compounds;
mod errors;
mod ext;
mod ffi_calls;
mod ffi_converters;
mod func_names;
mod functions;
mod into_ir;
mod objects;
mod primitives;
mod records;
mod util;

use crate::interface::ComponentInterface;
use bindings_ir::ir;
use ext::*;
use ffi_converters::*;
use into_ir::IntoIr;
use util::*;

pub(crate) fn generate_module(ci: &ComponentInterface, cdylib_name: &str) -> ir::Module {
    let mut module = ir::Module::new();
    module.add_buffer_stream_class("RustBufferStream");
    compounds::build_module(&mut module, ci);
    errors::build_module(&mut module, ci);
    ffi_calls::build_module(&mut module, ci, cdylib_name);
    functions::build_module(&mut module, ci);
    objects::build_module(&mut module, ci);
    primitives::build_module(&mut module, ci);
    records::build_module(&mut module, ci);
    module
}
