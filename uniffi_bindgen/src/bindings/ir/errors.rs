/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;

pub(super) fn build_module(module: &mut Module, ci: &interface::ComponentInterface) {
    for error in ci.error_definitions() {
        module.add_exception_base(exception_base_def(error.name()));
        for variant in error.variants() {
            module.add_exception(ExceptionDef {
                name: variant.name().into(),
                parent: error.name().into(),
                fields: variant.fields().into_ir(),
                ..ExceptionDef::default()
            });
        }
        add_error_write_func(module, error);
        add_error_allocation_size_func(module, error);
        add_error_read_func(module, error);
        add_rust_buffer_lift_and_lower_funcs(module, ci, &error.type_());
    }
}

fn add_error_read_func(module: &mut Module, error: &interface::Error) {
    let mut read_body = vec![val("variant_num", int32(), buf::read_int32(ident("stream")))];
    for (i, variant) in error.variants().into_iter().enumerate() {
        read_body.push(if_(
            eq(lit::int32((i + 1) as i32), ident("variant_num")),
            [return_(create_exception(
                variant.name(),
                variant
                    .fields()
                    .into_iter()
                    .map(|f| f.type_().call_read(ident("stream"))),
            ))],
        ));
    }
    read_body.push(raise_internal_exception(string::concat([
        lit::string(format!("{}.read: invalid variant num: ", error.name())),
        ident("variant_num"),
    ])));
    add_read_func(module, &error.type_(), read_body);
}

fn add_error_write_func(module: &mut Module, error: &interface::Error) {
    let write_variant = |variant_num, variant: &interface::Variant| {
        let mut block = vec![buf::write_int32(
            ident("stream"),
            lit::int32(variant_num as i32),
        )];
        for f in variant.fields() {
            block.push(
                f.type_()
                    .call_write(ident("stream"), get(ident("value"), f.name()))
                    .into_statement(),
            );
        }
        block
    };

    add_write_func(
        module,
        &error.type_(),
        error
            .variants()
            .into_iter()
            .enumerate()
            .map(|(i, variant)| {
                if_(
                    is_instance(ident("value"), variant.name()),
                    write_variant(i + 1, variant),
                )
            }),
    );
}

fn add_error_allocation_size_func(module: &mut Module, error: &interface::Error) {
    let mut body = vec![];
    for variant in error.variants() {
        body.push(if_(
            is_instance(ident("value"), variant.name()),
            [return_(
                variant
                    .fields()
                    .iter()
                    .map(|f| {
                        f.type_()
                            .call_allocation_size(get(ident("value"), f.name()))
                    })
                    .fold(
                        // 4 bytes for the i32 to specify the variast
                        lit::int32(4),
                        // Then add the size of each individual field for the variant
                        |a, b| add(int32(), a, b),
                    ),
            )],
        ));
    }
    body.push(raise_internal_exception(lit::string(format!(
        "{}.allocation_size: unknown variant",
        error.name()
    ))));
    add_allocation_size_func(module, &error.type_(), body);
}
