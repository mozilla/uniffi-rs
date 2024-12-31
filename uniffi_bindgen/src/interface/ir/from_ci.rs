/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};

use uniffi_meta::LiteralMetadata;

use crate::interface::{
    self, callbacks,
    ir::{
        sort::{sort_ffi_definitions, sort_types},
        *,
    },
};

impl TryFrom<interface::ComponentInterface> for BindingsIr {
    type Error = anyhow::Error;

    fn try_from(ci: interface::ComponentInterface) -> Result<Self> {
        let ffi_definitions = sort_ffi_definitions(ci.ffi_definitions().map(map_ffi_definition));
        let types = sort_types(
            ci.iter_types()
                .cloned()
                .map(|ty| map_type_definition(&ci, ty))
                .collect::<Result<Vec<_>>>()?,
        );
        // No need to sort functions, since they don't depend on other functions.
        let functions = ci
            .function_definitions()
            .iter()
            .cloned()
            .map(|func| map_func_definition(&ci, func))
            .collect();
        let globals = GlobalNodes {
            ffi_rustbuffer_alloc: map_ffi_func_ref(ci.ffi_rustbuffer_alloc()),
            ffi_rustbuffer_reserve: map_ffi_func_ref(ci.ffi_rustbuffer_reserve()),
            ffi_rustbuffer_free: map_ffi_func_ref(ci.ffi_rustbuffer_free()),
            ffi_uniffi_contract_version: map_ffi_func_ref(ci.ffi_uniffi_contract_version()),
            callback_interface_free_type: map_ffi_type(interface::FfiType::Callback(
                "CallbackInterfaceFree".to_string(),
            )),
            string_type: map_type(&ci, uniffi_meta::Type::String),
            contract_version: ci.uniffi_contract_version(),
        };
        let checksum_checks = ci
            .iter_checksums()
            .map(|(fn_name, checksum)| ChecksumCheck {
                func: name_to_ffi_func_ref(fn_name),
                checksum,
            })
            .collect();
        Ok(Self {
            namespace: ci.namespace().to_string(),
            crate_docstring: ci.types.namespace_docstring,
            globals,
            checksum_checks,
            ffi_definitions,
            type_definitions: types,
            functions,
        })
    }
}

fn map_ffi_definition(ffi_def: interface::FfiDefinition) -> FfiDefinition {
    match ffi_def {
        interface::FfiDefinition::Struct(s) => FfiDefinition::Struct(map_ffi_struct(s)),
        interface::FfiDefinition::Function(f) => FfiDefinition::Function(map_ffi_func(f)),
        interface::FfiDefinition::CallbackFunction(c) => {
            FfiDefinition::FunctionType(map_ffi_func_type(c))
        }
    }
}

fn map_ffi_struct(ffi_struct: interface::FfiStruct) -> FfiStruct {
    FfiStruct {
        name: ffi_struct.name,
        fields: ffi_struct.fields.into_iter().map(map_ffi_field).collect(),
    }
}

fn map_ffi_func(ffi_func: interface::FfiFunction) -> FfiFunction {
    FfiFunction {
        name: ffi_func.name,
        is_async: ffi_func.is_async,
        arguments: ffi_func
            .arguments
            .into_iter()
            .map(map_ffi_argument)
            .collect(),
        return_type: ffi_func.return_type.map(map_ffi_type),
        has_rust_call_status_arg: ffi_func.has_rust_call_status_arg,
        is_object_free_function: ffi_func.is_object_free_function,
    }
}

fn map_ffi_func_ref(ffi_func: interface::FfiFunction) -> FfiFunctionRef {
    FfiFunctionRef {
        name: ffi_func.name,
        is_object_free_function: ffi_func.is_object_free_function,
    }
}

fn map_ffi_func_type(callback: interface::FfiCallbackFunction) -> FfiFunctionType {
    FfiFunctionType {
        name: callback.name,
        arguments: callback
            .arguments
            .into_iter()
            .map(map_ffi_argument)
            .collect(),
        return_type: callback.return_type.map(map_ffi_type),
        has_rust_call_status_arg: callback.has_rust_call_status_arg,
    }
}

fn map_func_definition(ci: &interface::ComponentInterface, func: interface::Function) -> Function {
    let callable = map_callable(ci, &func, CallableKind::Function);
    Function {
        name: func.name,
        module_path: func.module_path,
        docstring: func.docstring,
        callable,
    }
}

fn map_callable(
    ci: &interface::ComponentInterface,
    callable: impl interface::Callable,
    kind: CallableKind,
) -> Callable {
    let async_data = callable.is_async().then(|| AsyncData {
        ffi_rust_future_poll: name_to_ffi_func_ref(callable.ffi_rust_future_poll(ci)),
        ffi_rust_future_complete: name_to_ffi_func_ref(callable.ffi_rust_future_complete(ci)),
        ffi_rust_future_free: name_to_ffi_func_ref(callable.ffi_rust_future_free(ci)),
        foreign_future_result_type: map_ffi_type(interface::FfiType::Struct(
            callbacks::foreign_future_ffi_result_struct(
                callable.return_type().map(interface::FfiType::from),
            )
            .name,
        )),
    });
    let arguments = callable
        .arguments()
        .into_iter()
        .map(|a| map_argument(ci, a.clone()))
        .collect();
    Callable {
        kind,
        async_data,
        arguments,
        return_type: callable.return_type().map(|ty| map_type(ci, ty)),
        throws_type: callable.throws_type().map(|ty| map_type(ci, ty)),
        ffi_func: map_ffi_func_ref(callable.ffi_func().clone()),
    }
}

fn map_type_definition(
    ci: &interface::ComponentInterface,
    ty: uniffi_meta::Type,
) -> Result<TypeDefinition> {
    Ok(match ty {
        uniffi_meta::Type::UInt8
        | uniffi_meta::Type::Int8
        | uniffi_meta::Type::UInt16
        | uniffi_meta::Type::Int16
        | uniffi_meta::Type::UInt32
        | uniffi_meta::Type::Int32
        | uniffi_meta::Type::UInt64
        | uniffi_meta::Type::Int64
        | uniffi_meta::Type::Float32
        | uniffi_meta::Type::Float64
        | uniffi_meta::Type::Boolean
        | uniffi_meta::Type::String
        | uniffi_meta::Type::Bytes
        | uniffi_meta::Type::Timestamp
        | uniffi_meta::Type::Duration => TypeDefinition::Simple(map_type(ci, ty)),
        uniffi_meta::Type::Optional { ref inner_type } => TypeDefinition::Optional(OptionalType {
            inner: map_type(ci, inner_type),
            self_type: map_type(ci, ty),
        }),
        uniffi_meta::Type::Sequence { ref inner_type } => TypeDefinition::Sequence(SequenceType {
            inner: map_type(ci, inner_type),
            self_type: map_type(ci, ty),
        }),
        uniffi_meta::Type::Map {
            ref key_type,
            ref value_type,
        } => TypeDefinition::Map(MapType {
            key: map_type(ci, key_type),
            value: map_type(ci, value_type),
            self_type: map_type(ci, ty),
        }),
        uniffi_meta::Type::Object { name, .. } => {
            let def = ci
                .get_object_definition(&name)
                .context("missing object definition")?
                .clone();
            let self_type = map_type(ci, &def);
            let vtable = def.has_callback_interface().then(|| {
                let ffi_type = def
                    .vtable()
                    .expect("has_callback_interface() true, but vtable() is None");
                let ffi_init_callback = def
                    .ffi_init_callback
                    .clone()
                    .expect("has_callback_interface() true, but Object::ffi_init_callback is None");
                let methods = def
                    .vtable_methods()
                    .into_iter()
                    .map(|(ffi_callback, method)| VTableMethod {
                        ffi_type: map_ffi_type(interface::FfiType::Callback(ffi_callback.name)),
                        name: method.name.clone(),
                        callable: map_callable(ci, method, CallableKind::VTableMethod),
                    })
                    .collect();
                VTable {
                    name: def.name.clone(),
                    ffi_type: map_ffi_type(ffi_type),
                    ffi_init_callback: map_ffi_func_ref(ffi_init_callback),
                    methods,
                }
            });
            TypeDefinition::Interface(Interface {
                name: def.name,
                imp: def.imp,
                module_path: def.module_path,
                remote: def.remote,
                constructors: def
                    .constructors
                    .into_iter()
                    .map(|c| map_constructor(ci, &self_type, c))
                    .collect(),
                methods: def
                    .methods
                    .into_iter()
                    .map(|meth| map_method(ci, &self_type, meth))
                    .collect(),
                uniffi_traits: def
                    .uniffi_traits
                    .into_iter()
                    .map(|ut| map_uniffi_trait(ci, &self_type, ut))
                    .collect(),
                trait_impls: def.trait_impls,
                vtable,
                ffi_clone: map_ffi_func_ref(def.ffi_func_clone),
                ffi_free: map_ffi_func_ref(def.ffi_func_free),
                docstring: def.docstring,
                self_type,
            })
        }
        uniffi_meta::Type::Record { name, .. } => {
            let def = ci
                .get_record_definition(&name)
                .context("missing record definition")?
                .clone();
            let self_type = map_type(ci, &def);
            TypeDefinition::Record(Record {
                name: def.name,
                module_path: def.module_path,
                remote: def.remote,
                fields: def.fields.into_iter().map(|f| map_field(ci, f)).collect(),
                docstring: def.docstring,
                self_type,
            })
        }
        uniffi_meta::Type::Enum { name, .. } => {
            let def = ci
                .get_enum_definition(&name)
                .context("missing enum definition")?
                .clone();
            let self_type = map_type(ci, &def);
            let discriminents = def.variant_discr_iter().collect::<Result<Vec<_>>>()?;
            let variants = def
                .variants
                .into_iter()
                .zip(discriminents)
                .map(|(variant, discr)| Variant {
                    name: variant.name,
                    fields: variant
                        .fields
                        .into_iter()
                        .map(|f| map_field(ci, f))
                        .collect(),
                    enum_shape: def.shape.clone(),
                    discr: map_literal(ci, discr),
                    docstring: variant.docstring,
                })
                .collect();

            TypeDefinition::Enum(Enum {
                name: def.name,
                module_path: def.module_path,
                remote: def.remote,
                discr_type: def.discr_type.map(|ty| map_type(ci, ty)),
                non_exhaustive: def.non_exhaustive,
                shape: def.shape,
                variants,
                docstring: def.docstring,
                self_type,
            })
        }
        uniffi_meta::Type::CallbackInterface { name, .. } => {
            let def = ci
                .get_callback_interface_definition(&name)
                .context("missing callback interface definition")?
                .clone();
            let self_type = map_type(ci, &def);
            let methods = def
                .vtable_methods()
                .into_iter()
                .map(|(ffi_callback, method)| VTableMethod {
                    ffi_type: map_ffi_type(interface::FfiType::Callback(ffi_callback.name)),
                    name: method.name.clone(),
                    callable: map_callable(ci, method, CallableKind::VTableMethod),
                })
                .collect();
            let vtable = VTable {
                name: def.name.clone(),
                ffi_type: map_ffi_type(def.vtable()),
                ffi_init_callback: map_ffi_func_ref(def.ffi_init_callback.clone()),
                methods,
            };

            TypeDefinition::CallbackInterface(CallbackInterface {
                name: def.name,
                module_path: def.module_path,
                docstring: def.docstring,
                methods: def
                    .methods
                    .into_iter()
                    .map(|meth| map_method(ci, &self_type, meth))
                    .collect(),
                vtable,
                self_type,
            })
        }
        uniffi_meta::Type::Custom {
            ref name,
            ref module_path,
            ref builtin,
        } => TypeDefinition::Custom(CustomType {
            name: name.clone(),
            module_path: module_path.clone(),
            builtin: map_type(ci, builtin),
            self_type: map_type(ci, ty),
        }),
        uniffi_meta::Type::External {
            ref name,
            ref module_path,
            ref namespace,
            kind,
        } => TypeDefinition::External(ExternalType {
            name: name.clone(),
            module_path: module_path.clone(),
            namespace: namespace.clone(),
            kind,
            self_type: map_type(ci, ty),
        }),
    })
}

fn map_constructor(
    ci: &interface::ComponentInterface,
    interface: &Type,
    cons: interface::Constructor,
) -> Constructor {
    let primary = cons.is_primary_constructor();
    let callable = map_callable(ci, &cons, CallableKind::Constructor { primary });
    Constructor {
        name: cons.name,
        docstring: cons.docstring,
        interface: interface.clone(),
        primary,
        object_module_path: cons.object_module_path,
        callable,
    }
}

fn map_method(
    ci: &interface::ComponentInterface,
    interface: &Type,
    meth: interface::Method,
) -> Method {
    let callable = map_callable(ci, &meth, CallableKind::Method);
    Method {
        name: meth.name,
        interface: interface.clone(),
        object_module_path: meth.object_module_path,
        object_impl: meth.object_impl,
        callable,
        docstring: meth.docstring,
    }
}

fn map_uniffi_trait(
    ci: &interface::ComponentInterface,
    interface: &Type,
    ut: interface::UniffiTrait,
) -> UniffiTrait {
    match ut {
        interface::UniffiTrait::Debug { fmt } => UniffiTrait::Debug {
            fmt: map_method(ci, interface, fmt),
        },
        interface::UniffiTrait::Display { fmt } => UniffiTrait::Display {
            fmt: map_method(ci, interface, fmt),
        },
        interface::UniffiTrait::Eq { eq, ne } => UniffiTrait::Eq {
            eq: map_method(ci, interface, eq),
            ne: map_method(ci, interface, ne),
        },
        interface::UniffiTrait::Hash { hash } => UniffiTrait::Hash {
            hash: map_method(ci, interface, hash),
        },
    }
}

fn map_argument(ci: &interface::ComponentInterface, arg: interface::Argument) -> Argument {
    Argument {
        ty: map_type(ci, &arg),
        name: arg.name,
        by_ref: arg.by_ref,
        optional: arg.optional,
        default: arg.default.map(|l| map_literal(ci, l)),
    }
}

fn map_field(ci: &interface::ComponentInterface, field: interface::Field) -> Field {
    Field {
        ty: map_type(ci, &field),
        name: field.name,
        default: field.default.map(|l| map_literal(ci, l)),
        docstring: field.docstring,
    }
}

fn map_type(ci: &interface::ComponentInterface, ty: impl interface::AsType) -> Type {
    let ty = ty.as_type();
    let ffi_type = map_ffi_type((&ty).into());
    let is_used_as_error = match ty.name() {
        None => false,
        Some(name) => ci.errors.contains(name),
    };
    Type {
        kind: ty,
        ffi_type,
        is_used_as_error,
    }
}

fn map_literal(ci: &interface::ComponentInterface, lit: LiteralMetadata) -> Literal {
    match lit {
        LiteralMetadata::Boolean(b) => Literal::Boolean(b),
        LiteralMetadata::String(s) => Literal::String(s),
        LiteralMetadata::UInt(val, radix, ty) => Literal::UInt(val, radix, map_type(ci, ty)),
        LiteralMetadata::Int(val, radix, ty) => Literal::Int(val, radix, map_type(ci, ty)),
        LiteralMetadata::Float(repr, ty) => Literal::Float(repr, map_type(ci, ty)),
        LiteralMetadata::Enum(repr, ty) => Literal::Enum(repr, map_type(ci, ty)),
        LiteralMetadata::EmptySequence => Literal::EmptySequence,
        LiteralMetadata::EmptyMap => Literal::EmptyMap,
        LiteralMetadata::None => Literal::None,
        LiteralMetadata::Some { inner } => Literal::Some {
            inner: Box::new(map_literal(ci, *inner)),
        },
    }
}

fn map_ffi_field(field: interface::FfiField) -> FfiField {
    FfiField {
        name: field.name,
        ty: map_ffi_type(field.type_),
    }
}

fn map_ffi_argument(arg: interface::FfiArgument) -> FfiArgument {
    FfiArgument {
        name: arg.name,
        ty: map_ffi_type(arg.type_),
    }
}

fn map_ffi_type(ty: interface::FfiType) -> FfiType {
    match ty {
        interface::FfiType::UInt8 => FfiType::UInt8,
        interface::FfiType::Int8 => FfiType::Int8,
        interface::FfiType::UInt16 => FfiType::UInt16,
        interface::FfiType::Int16 => FfiType::Int16,
        interface::FfiType::UInt32 => FfiType::UInt32,
        interface::FfiType::Int32 => FfiType::Int32,
        interface::FfiType::UInt64 => FfiType::UInt64,
        interface::FfiType::Int64 => FfiType::Int64,
        interface::FfiType::Float32 => FfiType::Float32,
        interface::FfiType::Float64 => FfiType::Float64,
        interface::FfiType::RustArcPtr(name) => FfiType::RustArcPtr(name),
        interface::FfiType::RustBuffer(meta) => {
            FfiType::RustBuffer(meta.map(|m| ExternalFfiMetadata {
                namespace: m.namespace,
                module_path: m.module_path,
            }))
        }
        interface::FfiType::ForeignBytes => FfiType::ForeignBytes,
        interface::FfiType::Callback(name) => FfiType::FunctionPointer(name),
        interface::FfiType::Struct(name) => FfiType::Struct(name),
        interface::FfiType::Handle => FfiType::Handle,
        interface::FfiType::RustCallStatus => FfiType::RustCallStatus,
        interface::FfiType::Reference(inner_type) => {
            FfiType::Reference(Box::new(map_ffi_type(*inner_type)))
        }
        interface::FfiType::MutReference(inner_type) => {
            FfiType::MutReference(Box::new(map_ffi_type(*inner_type)))
        }
        interface::FfiType::VoidPointer => FfiType::VoidPointer,
    }
}

fn name_to_ffi_func_ref(name: String) -> FfiFunctionRef {
    FfiFunctionRef {
        name,
        is_object_free_function: false,
    }
}
