/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn build_module(module: &mut Module, ci: &interface::ComponentInterface) {
    for rec in ci.record_definitions() {
        let type_ = rec.type_();
        let fields = rec.fields();

        module.add_data_class(DataClassDef {
            vis: public(),
            name: rec.name().into(),
            fields: fields.clone().into_ir(),
        });
        add_allocation_size_func(
            module,
            &type_,
            [return_(
                fields
                    .iter()
                    .map(|f| {
                        f.type_()
                            .call_allocation_size(get(ident("value"), f.name()))
                    })
                    .reduce(|a, b| add(int32(), a, b))
                    .unwrap_or(lit::int32(0)),
            )],
        );
        add_read_func(
            module,
            &type_,
            [return_(create_data_class(
                rec.name(),
                fields.iter().map(|f| f.type_().call_read(ident("stream"))),
            ))],
        );
        add_write_func(
            module,
            &type_,
            fields.iter().map(|f| {
                f.type_()
                    .call_write(ident("stream"), get(ident("value"), f.name()))
                    .into_statement()
            }),
        );
        add_rust_buffer_lift_and_lower_funcs(module, ci, &type_);
    }
}
