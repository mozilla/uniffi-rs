/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//mod config;

use crate::ir::{general_pipeline, Pipeline};

pub fn python_pipeline() -> impl Pipeline {
    general_pipeline()
    //        .pass(config::pass)
    //.pass(python_module_path::pass)
    //.pass(names::pass)
    //.pass(docstrings::pass)
    //.pass(interface_protocols::pass)
    //.pass(interface_base_classes::pass)
    //.pass(constructors::pass)
    //.pass(vtable::pass)
    //.pass(type_names::pass)
    //.pass(ffi_type_names::pass)
    //.pass(literals::pass)
}
