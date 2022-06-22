/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use crate::interface;
use bindings_ir::ir::*;
use heck::ToSnakeCase;
use std::iter::once;

pub(super) fn build_module(module: &mut Module, ci: &interface::ComponentInterface) {
    for obj in ci.object_definitions() {
        let type_ = obj.type_();
        add_class_def(module, obj);
        add_allocation_size_func(module, &type_, [return_(pointer_size())]);
        add_lift_func(
            module,
            &type_,
            [return_(create_class(obj.name(), [ident("value")]))],
        );
        add_read_func(
            module,
            &type_,
            [return_(create_class(
                obj.name(),
                [buf::read_pointer(obj.name(), ident("stream"))],
            ))],
        );
        add_lower_func(module, &type_, [return_(ident("value.ptr"))]);
        add_write_func(
            module,
            &type_,
            [buf::write_pointer(
                obj.name(),
                ident("stream"),
                ident("value.ptr"),
            )],
        );
    }
}

fn add_class_def(module: &mut Module, obj: &interface::Object) {
    let mut class_def = ClassDef {
        vis: public(),
        name: obj.name().into(),
        fields: vec![field("ptr", pointer(obj.name()))],
        methods: obj
            .methods()
            .into_iter()
            .map(|meth| define_method(obj, meth))
            .collect(),
        destructor: Destructor {
            body: block([
                empty_rust_status_var("status"),
                ffi_call(
                    obj.ffi_object_free().name(),
                    [ident("ptr"), ident("status")],
                )
                .into_statement(),
                // Don't check status, if the call fails we can't really do anything about it since
                // we're in a destructor.
            ]),
        }
        .into(),
        ..ClassDef::default()
    };
    if let Some(cons) = obj.primary_constructor() {
        add_constructor(module, &mut class_def, obj, &cons);
    }
    for cons in obj.constructors() {
        add_secondary_constructor(&mut class_def, obj, &cons);
    }
    module.add_class(class_def);
}

fn add_constructor(
    module: &mut Module,
    class_def: &mut ClassDef,
    obj: &interface::Object,
    cons: &interface::Constructor,
) {
    // Define a helper function to make the FFI call, check for errors, etc.
    let helper_name = format!("uniffi_construct_{}", obj.name().to_snake_case());
    module.add_function(FunctionDef {
        name: helper_name.clone().into(),
        vis: private(),
        args: cons.arguments().into_ir(),
        return_type: pointer(obj.name()).into(),
        throws: None,
        body: block([
            empty_rust_status_var("status"),
            val(
                "ptr",
                pointer(obj.name()),
                ffi_call(
                    cons.ffi_func().name(),
                    cons.arguments()
                        .into_iter()
                        .map(|a| a.type_().call_lower(ident(a.name())))
                        .chain(once(ident("status"))),
                ),
            ),
            call_throw_if_error("status"),
            return_(ident("ptr")),
        ]),
    });
    // Define the constructor using the helper func
    class_def.constructor = Constructor {
        vis: public(),
        args: cons.arguments().into_ir(),
        initializers: vec![call(
            helper_name,
            cons.arguments().into_iter().map(|a| ident(a.name())),
        )],
    }
    .into();
}

fn add_secondary_constructor(
    class_def: &mut ClassDef,
    obj: &interface::Object,
    cons: &interface::Constructor,
) {
    // Implement secondary constructors as static factory methods
    class_def.methods.push(Method {
        vis: public(),
        name: cons.name().into(),
        method_type: MethodType::Static,
        args: cons.arguments().into_ir(),
        return_type: object(obj.name()).into(),
        throws: None,
        body: block([
            empty_rust_status_var("status"),
            val(
                "ptr",
                pointer(obj.name()),
                ffi_call(
                    cons.ffi_func().name(),
                    cons.arguments()
                        .into_iter()
                        .map(|a| a.type_().call_lower(ident(a.name())))
                        .chain(once(ident("status"))),
                ),
            ),
            call_throw_if_error("status"),
            return_(create_class(obj.name(), [ident("ptr")])),
        ]),
    });
}

fn define_method(obj: &interface::Object, meth: &interface::Method) -> Method {
    let mut ffi_call_args = vec![obj.type_().call_lower(this())];
    for arg in meth.arguments() {
        ffi_call_args.push(arg.type_().call_lower(ident(arg.name())))
    }
    ffi_call_args.push(ref_mut("uniffi_call_status"));

    let make_ffi_call = ffi_call(meth.ffi_func().name(), ffi_call_args);

    // Create statements to make the FFI call and return the result on success
    let (ffi_call_stmt, success_return) = match meth.return_type() {
        None => (make_ffi_call.into_statement(), return_void()),
        Some(return_type) => (
            val(
                "uniffi_call_return",
                return_type.clone().into_ir(),
                return_type.call_lift(make_ffi_call),
            ),
            return_(ident("uniffi_call_return")),
        ),
    };

    let body = block([
        var(
            "uniffi_call_status",
            cstruct("RustCallStatus"),
            create_cstruct(
                "RustCallStatus",
                [
                    lit::int32(0),
                    create_cstruct("RustBuffer", [lit::int32(0), lit::int32(0), lit::null()]),
                ],
            ),
        ),
        ffi_call_stmt,
        // Throw an error if `uniffi_call_status` indicates we should
        match meth.throws_type() {
            Some(error_type) => call_throw_if_error_with_type(&error_type, "uniffi_call_status"),
            None => call_throw_if_error("uniffi_call_status"),
        },
        // If not, return success
        success_return,
    ]);

    Method {
        vis: public(),
        method_type: MethodType::Normal,
        name: meth.name().into(),
        args: meth.arguments().into_ir(),
        return_type: meth.return_type().into_ir(),
        throws: meth.throws_name().map(ClassName::from),
        body,
    }
}
