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
    Callable,
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
        let globals = GlobalDefinitions {
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
        lang_data: LanguageData::default(),
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
        return_type: FfiReturnType {
            ty: ffi_func.return_type.map(map_ffi_type),
            lang_data: LanguageData::default(),
        },
        has_rust_call_status_arg: ffi_func.has_rust_call_status_arg,
        is_object_free_function: ffi_func.is_object_free_function,
        lang_data: LanguageData::default(),
    }
}

fn map_ffi_func_ref(ffi_func: interface::FfiFunction) -> FfiFunctionRef {
    FfiFunctionRef {
        name: ffi_func.name,
        is_object_free_function: ffi_func.is_object_free_function,
        lang_data: LanguageData::default(),
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
        return_type: FfiReturnType {
            ty: callback.return_type.map(map_ffi_type),
            lang_data: LanguageData::default(),
        },
        has_rust_call_status_arg: callback.has_rust_call_status_arg,
        lang_data: LanguageData::default(),
    }
}

fn map_func_definition(ci: &interface::ComponentInterface, func: interface::Function) -> Function {
    let checksum = func.checksum();
    let async_data = func.is_async.then(|| AsyncData {
        ffi_rust_future_poll: name_to_ffi_func_ref(func.ffi_rust_future_poll(ci)),
        ffi_rust_future_complete: name_to_ffi_func_ref(func.ffi_rust_future_complete(ci)),
        ffi_rust_future_free: name_to_ffi_func_ref(func.ffi_rust_future_free(ci)),
        foreign_future_result_type: map_ffi_type(interface::FfiType::Struct(
            callbacks::foreign_future_ffi_result_struct(
                func.return_type.as_ref().map(interface::FfiType::from),
            )
            .name,
        )),
    });
    Function {
        name: func.name,
        module_path: func.module_path,
        async_data,
        arguments: func
            .arguments
            .into_iter()
            .map(|a| map_argument(ci, a))
            .collect(),
        ffi_func: map_ffi_func_ref(func.ffi_func),
        return_type: ReturnType {
            ty: func.return_type.map(|ty| map_type(ci, ty)),
            lang_data: LanguageData::default(),
        },
        throws_type: ThrowsType {
            ty: func.throws.map(|ty| map_type(ci, ty)),
            lang_data: LanguageData::default(),
        },
        checksum_func: name_to_ffi_func_ref(func.checksum_fn_name),
        checksum,
        docstring: func.docstring,
        lang_data: LanguageData::default(),
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
        | uniffi_meta::Type::Duration
        | uniffi_meta::Type::Optional { .. }
        | uniffi_meta::Type::Sequence { .. }
        | uniffi_meta::Type::Map { .. } => TypeDefinition::Builtin(map_type(ci, ty)),
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
                let (method_ffi_types, methods) = def
                    .vtable_methods()
                    .into_iter()
                    .map(|(ffi_callback, method)| {
                        (
                            map_ffi_type(interface::FfiType::Callback(ffi_callback.name)),
                            map_method(ci, &self_type, method),
                        )
                    })
                    .unzip();
                VTable {
                    name: def.name.clone(),
                    ffi_type: map_ffi_type(ffi_type),
                    ffi_init_callback: map_ffi_func_ref(ffi_init_callback),
                    methods,
                    method_ffi_types,
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
                lang_data: LanguageData::default(),
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
                lang_data: LanguageData::default(),
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
                    lang_data: LanguageData::default(),
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
                lang_data: LanguageData::default(),
            })
        }
        uniffi_meta::Type::CallbackInterface { name, .. } => {
            let def = ci
                .get_callback_interface_definition(&name)
                .context("missing callback interface definition")?
                .clone();
            let self_type = map_type(ci, &def);
            let (method_ffi_types, methods) = def
                .vtable_methods()
                .into_iter()
                .map(|(ffi_callback, method)| {
                    (
                        map_ffi_type(interface::FfiType::Callback(ffi_callback.name)),
                        map_method(ci, &self_type, method),
                    )
                })
                .unzip();
            let vtable = VTable {
                name: def.name.clone(),
                ffi_type: map_ffi_type(def.vtable()),
                ffi_init_callback: map_ffi_func_ref(def.ffi_init_callback.clone()),
                methods,
                method_ffi_types,
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
                lang_data: LanguageData::default(),
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
            lang_data: LanguageData::default(),
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
            lang_data: LanguageData::default(),
        }),
    })
}

fn map_constructor(
    ci: &interface::ComponentInterface,
    interface: &Type,
    cons: interface::Constructor,
) -> Constructor {
    let checksum = cons.checksum();
    let async_data = cons.is_async().then(|| AsyncData {
        ffi_rust_future_poll: name_to_ffi_func_ref(cons.ffi_rust_future_poll(ci)),
        ffi_rust_future_complete: name_to_ffi_func_ref(cons.ffi_rust_future_complete(ci)),
        ffi_rust_future_free: name_to_ffi_func_ref(cons.ffi_rust_future_free(ci)),
        foreign_future_result_type: map_ffi_type(interface::FfiType::Struct(
            callbacks::foreign_future_ffi_result_struct(Some(interface::FfiType::RustArcPtr(
                cons.object_name.clone(),
            )))
            .name,
        )),
    });
    let primary = cons.is_primary_constructor();
    Constructor {
        name: cons.name,
        interface: interface.clone(),
        primary,
        object_module_path: cons.object_module_path,
        async_data,
        arguments: cons
            .arguments
            .into_iter()
            .map(|a| map_argument(ci, a))
            .collect(),
        return_type: ReturnType {
            ty: Some(interface.clone()),
            lang_data: LanguageData::default(),
        },
        ffi_func: map_ffi_func_ref(cons.ffi_func),
        docstring: cons.docstring,
        throws_type: ThrowsType {
            ty: cons.throws.map(|ty| map_type(ci, ty)),
            lang_data: LanguageData::default(),
        },
        checksum_func: name_to_ffi_func_ref(cons.checksum_fn_name),
        checksum,
        lang_data: LanguageData::default(),
    }
}

fn map_method(
    ci: &interface::ComponentInterface,
    interface: &Type,
    meth: interface::Method,
) -> Method {
    let checksum = meth.checksum();
    let async_data = meth.is_async.then(|| AsyncData {
        ffi_rust_future_poll: name_to_ffi_func_ref(meth.ffi_rust_future_poll(ci)),
        ffi_rust_future_complete: name_to_ffi_func_ref(meth.ffi_rust_future_complete(ci)),
        ffi_rust_future_free: name_to_ffi_func_ref(meth.ffi_rust_future_free(ci)),
        foreign_future_result_type: map_ffi_type(interface::FfiType::Struct(
            callbacks::foreign_future_ffi_result_struct(
                meth.return_type.as_ref().map(interface::FfiType::from),
            )
            .name,
        )),
    });
    Method {
        name: meth.name,
        interface: interface.clone(),
        object_module_path: meth.object_module_path,
        object_impl: meth.object_impl,
        async_data,
        arguments: meth
            .arguments
            .into_iter()
            .map(|a| map_argument(ci, a))
            .collect(),
        ffi_func: map_ffi_func_ref(meth.ffi_func),
        return_type: ReturnType {
            ty: meth.return_type.map(|ty| map_type(ci, ty)),
            lang_data: LanguageData::default(),
        },
        throws_type: ThrowsType {
            ty: meth.throws.map(|ty| map_type(ci, ty)),
            lang_data: LanguageData::default(),
        },
        docstring: meth.docstring,
        checksum_func: name_to_ffi_func_ref(meth.checksum_fn_name),
        checksum,
        lang_data: LanguageData::default(),
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
        lang_data: LanguageData::default(),
    }
}

fn map_field(ci: &interface::ComponentInterface, field: interface::Field) -> Field {
    Field {
        ty: map_type(ci, &field),
        name: field.name,
        default: field.default.map(|l| map_literal(ci, l)),
        docstring: field.docstring,
        lang_data: LanguageData::default(),
    }
}

fn map_type(ci: &interface::ComponentInterface, ty: impl interface::AsType) -> Type {
    let ty = ty.as_type();
    let ffi_type = map_ffi_type((&ty).into());
    let is_used_as_error = match ty.name() {
        None => false,
        Some(name) => ci.errors.contains(name),
    };
    let kind = match ty {
        uniffi_meta::Type::UInt8 => TypeKind::UInt8,
        uniffi_meta::Type::Int8 => TypeKind::Int8,
        uniffi_meta::Type::UInt16 => TypeKind::UInt16,
        uniffi_meta::Type::Int16 => TypeKind::Int16,
        uniffi_meta::Type::UInt32 => TypeKind::UInt32,
        uniffi_meta::Type::Int32 => TypeKind::Int32,
        uniffi_meta::Type::UInt64 => TypeKind::UInt64,
        uniffi_meta::Type::Int64 => TypeKind::Int64,
        uniffi_meta::Type::Float32 => TypeKind::Float32,
        uniffi_meta::Type::Float64 => TypeKind::Float64,
        uniffi_meta::Type::Boolean => TypeKind::Boolean,
        uniffi_meta::Type::String => TypeKind::String,
        uniffi_meta::Type::Bytes => TypeKind::Bytes,
        uniffi_meta::Type::Timestamp => TypeKind::Timestamp,
        uniffi_meta::Type::Duration => TypeKind::Duration,
        uniffi_meta::Type::Object {
            module_path,
            name,
            imp,
        } => TypeKind::Interface {
            module_path,
            name,
            imp,
        },
        uniffi_meta::Type::Record { module_path, name } => TypeKind::Record { module_path, name },
        uniffi_meta::Type::Enum { module_path, name } => TypeKind::Enum { module_path, name },
        uniffi_meta::Type::CallbackInterface { module_path, name } => {
            TypeKind::CallbackInterface { module_path, name }
        }
        uniffi_meta::Type::External {
            module_path,
            name,
            namespace,
            kind,
        } => TypeKind::External {
            module_path,
            name,
            namespace,
            kind,
        },
        uniffi_meta::Type::Custom {
            module_path,
            name,
            builtin,
        } => TypeKind::Custom {
            module_path,
            name,
            builtin: Box::new(map_type(ci, *builtin)),
        },
        uniffi_meta::Type::Optional { inner_type } => TypeKind::Optional {
            inner_type: Box::new(map_type(ci, *inner_type)),
        },
        uniffi_meta::Type::Sequence { inner_type } => TypeKind::Sequence {
            inner_type: Box::new(map_type(ci, *inner_type)),
        },
        uniffi_meta::Type::Map {
            key_type,
            value_type,
        } => TypeKind::Map {
            key_type: Box::new(map_type(ci, *key_type)),
            value_type: Box::new(map_type(ci, *value_type)),
        },
    };
    Type {
        kind,
        ffi_type,
        is_used_as_error,
        lang_data: LanguageData::default(),
    }
}

fn map_literal(ci: &interface::ComponentInterface, lit: LiteralMetadata) -> Literal {
    let kind = match lit {
        LiteralMetadata::Boolean(b) => LiteralKind::Boolean(b),
        LiteralMetadata::String(s) => LiteralKind::String(s),
        LiteralMetadata::UInt(val, radix, ty) => LiteralKind::UInt(val, radix, map_type(ci, ty)),
        LiteralMetadata::Int(val, radix, ty) => LiteralKind::Int(val, radix, map_type(ci, ty)),
        LiteralMetadata::Float(repr, ty) => LiteralKind::Float(repr, map_type(ci, ty)),
        LiteralMetadata::Enum(repr, ty) => LiteralKind::Enum(repr, map_type(ci, ty)),
        LiteralMetadata::EmptySequence => LiteralKind::EmptySequence,
        LiteralMetadata::EmptyMap => LiteralKind::EmptyMap,
        LiteralMetadata::None => LiteralKind::None,
        LiteralMetadata::Some { inner } => LiteralKind::Some {
            inner: Box::new(map_literal(ci, *inner)),
        },
    };
    Literal {
        kind,
        lang_data: LanguageData::default(),
    }
}

fn map_ffi_field(field: interface::FfiField) -> FfiField {
    FfiField {
        name: field.name,
        ty: map_ffi_type(field.type_),
        lang_data: LanguageData::default(),
    }
}

fn map_ffi_argument(arg: interface::FfiArgument) -> FfiArgument {
    FfiArgument {
        name: arg.name,
        ty: map_ffi_type(arg.type_),
        lang_data: LanguageData::default(),
    }
}

fn map_ffi_type(ty: interface::FfiType) -> FfiType {
    let kind = match ty {
        interface::FfiType::UInt8 => FfiTypeKind::UInt8,
        interface::FfiType::Int8 => FfiTypeKind::Int8,
        interface::FfiType::UInt16 => FfiTypeKind::UInt16,
        interface::FfiType::Int16 => FfiTypeKind::Int16,
        interface::FfiType::UInt32 => FfiTypeKind::UInt32,
        interface::FfiType::Int32 => FfiTypeKind::Int32,
        interface::FfiType::UInt64 => FfiTypeKind::UInt64,
        interface::FfiType::Int64 => FfiTypeKind::Int64,
        interface::FfiType::Float32 => FfiTypeKind::Float32,
        interface::FfiType::Float64 => FfiTypeKind::Float64,
        interface::FfiType::RustArcPtr(name) => FfiTypeKind::RustArcPtr(name),
        interface::FfiType::RustBuffer(meta) => {
            FfiTypeKind::RustBuffer(meta.map(|m| ExternalFfiMetadata {
                namespace: m.namespace,
                module_path: m.module_path,
            }))
        }
        interface::FfiType::ForeignBytes => FfiTypeKind::ForeignBytes,
        interface::FfiType::Callback(name) => FfiTypeKind::FunctionPointer(name),
        interface::FfiType::Struct(name) => FfiTypeKind::Struct(name),
        interface::FfiType::Handle => FfiTypeKind::Handle,
        interface::FfiType::RustCallStatus => FfiTypeKind::RustCallStatus,
        interface::FfiType::Reference(inner_type) => {
            FfiTypeKind::Reference(Box::new(map_ffi_type(*inner_type)))
        }
        interface::FfiType::MutReference(inner_type) => {
            FfiTypeKind::MutReference(Box::new(map_ffi_type(*inner_type)))
        }
        interface::FfiType::VoidPointer => FfiTypeKind::VoidPointer,
    };
    FfiType {
        kind,
        lang_data: LanguageData::default(),
    }
}

fn name_to_ffi_func_ref(name: String) -> FfiFunctionRef {
    FfiFunctionRef {
        name,
        is_object_free_function: false,
        lang_data: LanguageData::default(),
    }
}
