/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashSet};

use anyhow::{bail, Result};
use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use once_cell::sync::Lazy;

use super::*;
use crate::interface::ir;

/// Convert a general IR to a Python IR
pub fn convert_bindings_ir(general_ir: BindingsIr, config: Config) -> Result<PythonBindingsIr> {
    BindingsIrConverter::new(config).convert_bindings_ir(general_ir)
}

/// Struct that represents an in-process conversion
struct BindingsIrConverter {
    config: Config,
}

impl BindingsIrConverter {
    fn new(config: Config) -> Self {
        Self { config }
    }

    fn convert_bindings_ir(&self, general_ir: BindingsIr) -> Result<PythonBindingsIr> {
        // Create a mostly empty new `BindingsIr`
        let globals = GlobalNodes {
            ffi_rustbuffer_alloc: general_ir.globals.ffi_rustbuffer_alloc.name,
            ffi_rustbuffer_reserve: general_ir.globals.ffi_rustbuffer_reserve.name,
            ffi_rustbuffer_free: general_ir.globals.ffi_rustbuffer_free.name,
            ffi_uniffi_contract_version: general_ir.globals.ffi_uniffi_contract_version.name,
            callback_interface_free_type: self
                .ffi_type_name(general_ir.globals.callback_interface_free_type),
            string_type: self.convert_type(general_ir.globals.string_type),
            contract_version: general_ir.globals.contract_version,
        };
        let mut py_ir = PythonBindingsIr {
            namespace: general_ir.namespace,
            cdylib_name: self
                .config
                .cdylib_name
                .clone()
                .unwrap_or_else(|| "uniffi".to_string()),
            module_docstring: self.format_docstring(general_ir.crate_docstring),
            runtimes: Runtimes::default(),
            globals,
            ffi_definitions: vec![],
            protocols: vec![],
            type_definitions: vec![],
            functions: vec![],
            checksum_checks: vec![],
            imports: BTreeSet::new(),
            exports: vec![],
        };
        // Call the `process_*` methods to populate the new IR
        for ffi_def in general_ir.ffi_definitions {
            match ffi_def {
                ir::FfiDefinition::Function(f) => self.process_ffi_func(&mut py_ir, f),
                ir::FfiDefinition::FunctionType(f) => self.process_ffi_func_type(&mut py_ir, f),
                ir::FfiDefinition::Struct(s) => self.process_ffi_struct(&mut py_ir, s),
            }
        }
        for type_def in general_ir.type_definitions {
            match type_def {
                ir::TypeDefinition::Simple(t) => self.process_builtin(&mut py_ir, t)?,
                ir::TypeDefinition::Optional(o) => self.process_optional(&mut py_ir, o)?,
                ir::TypeDefinition::Sequence(o) => self.process_sequence(&mut py_ir, o)?,
                ir::TypeDefinition::Map(m) => self.process_map(&mut py_ir, m)?,
                ir::TypeDefinition::Record(r) => self.process_record(&mut py_ir, r)?,
                ir::TypeDefinition::Enum(e) => self.process_enum(&mut py_ir, e)?,
                ir::TypeDefinition::Interface(i) => self.process_interface(&mut py_ir, i)?,
                ir::TypeDefinition::CallbackInterface(c) => {
                    self.process_callback_interface(&mut py_ir, c)?
                }
                ir::TypeDefinition::Custom(c) => self.process_custom_type(&mut py_ir, c)?,
                ir::TypeDefinition::External(e) => self.process_external_type(&mut py_ir, e)?,
            }
        }
        for func in general_ir.functions {
            self.process_function(&mut py_ir, func)?;
        }
        for check in general_ir.checksum_checks {
            self.process_checksum_check(&mut py_ir, check);
        }
        Ok(py_ir)
    }

    fn process_ffi_func(&self, py_ir: &mut PythonBindingsIr, ffi_func: ir::FfiFunction) {
        let ffi_func = FfiFunction {
            name: ffi_func.name,
            is_async: ffi_func.is_async,
            arguments: self.convert_ffi_arguments(ffi_func.arguments),
            return_type: ffi_func
                .return_type
                .map(|ffi_type| self.ffi_type_name(ffi_type)),
            has_rust_call_status_arg: ffi_func.has_rust_call_status_arg,
        };
        py_ir
            .ffi_definitions
            .push(FfiDefinition::Function(ffi_func));
    }

    fn process_ffi_func_type(
        &self,
        py_ir: &mut PythonBindingsIr,
        ffi_func_type: ir::FfiFunctionType,
    ) {
        let ffi_func_type = FfiFunctionType {
            name: self.ffi_function_type_name(&ffi_func_type.name),
            arguments: self.convert_ffi_arguments(ffi_func_type.arguments),
            return_type: ffi_func_type
                .return_type
                .map(|ffi_type| self.ffi_type_name(ffi_type)),
            has_rust_call_status_arg: ffi_func_type.has_rust_call_status_arg,
        };
        py_ir
            .ffi_definitions
            .push(FfiDefinition::FunctionType(ffi_func_type));
    }

    fn process_ffi_struct(&self, py_ir: &mut PythonBindingsIr, st: ir::FfiStruct) {
        let ffi_struct = FfiStruct {
            name: self.ffi_struct_name(&st.name),
            fields: self.convert_ffi_fields(st.fields),
        };
        py_ir
            .ffi_definitions
            .push(FfiDefinition::Struct(ffi_struct));
    }

    fn process_checksum_check(&self, py_ir: &mut PythonBindingsIr, check: ir::ChecksumCheck) {
        py_ir.checksum_checks.push(ChecksumCheck {
            func: check.func.name,
            checksum: check.checksum,
        })
    }

    fn convert_ffi_arguments(&self, args: Vec<ir::FfiArgument>) -> Vec<FfiArgument> {
        args.into_iter()
            .map(|arg| FfiArgument {
                name: arg.name,
                ty: self.ffi_type_name(arg.ty),
            })
            .collect()
    }

    fn convert_ffi_fields(&self, fields: Vec<ir::FfiField>) -> Vec<FfiField> {
        fields
            .into_iter()
            .map(|field| FfiField {
                name: field.name,
                ty: self.ffi_type_name(field.ty),
            })
            .collect()
    }

    fn process_builtin(&self, py_ir: &mut PythonBindingsIr, ty: ir::Type) -> Result<()> {
        py_ir.type_definitions.push(match &ty.kind {
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
            | uniffi_meta::Type::Duration => TypeDefinition::Simple(self.convert_type(ty)),
            _ => bail!("Invalid builtin type: {ty:?}"),
        });
        Ok(())
    }

    fn process_optional(&self, py_ir: &mut PythonBindingsIr, opt: ir::OptionalType) -> Result<()> {
        py_ir
            .type_definitions
            .push(TypeDefinition::Optional(OptionalType {
                inner: self.convert_type(opt.inner),
                self_type: self.convert_type(opt.self_type),
            }));
        Ok(())
    }

    fn process_sequence(&self, py_ir: &mut PythonBindingsIr, seq: ir::SequenceType) -> Result<()> {
        py_ir
            .type_definitions
            .push(TypeDefinition::Sequence(SequenceType {
                inner: self.convert_type(seq.inner),
                self_type: self.convert_type(seq.self_type),
            }));
        Ok(())
    }

    fn process_map(&self, py_ir: &mut PythonBindingsIr, map: ir::MapType) -> Result<()> {
        py_ir.type_definitions.push(TypeDefinition::Map(MapType {
            key: self.convert_type(map.key),
            value: self.convert_type(map.value),
            self_type: self.convert_type(map.self_type),
        }));
        Ok(())
    }

    fn process_record(&self, py_ir: &mut PythonBindingsIr, rec: ir::Record) -> Result<()> {
        let rec = Record {
            name: self.class_name(&rec.name),
            fields: self.convert_fields(rec.fields)?,
            docstring: self.format_docstring(rec.docstring),
            self_type: self.convert_type(rec.self_type),
        };
        py_ir.exports.push(rec.name.clone());
        py_ir.type_definitions.push(TypeDefinition::Record(rec));
        Ok(())
    }

    fn process_enum(&self, py_ir: &mut PythonBindingsIr, en: ir::Enum) -> Result<()> {
        let enum_ = Enum {
            name: self.class_name(&en.name),
            discr_type: self.convert_opt_type(en.discr_type),
            variants: self.convert_variants(en.variants)?,
            shape: en.shape,
            docstring: self.format_docstring(en.docstring),
            self_type: self.convert_type(en.self_type),
        };
        py_ir.exports.push(enum_.name.clone());
        py_ir.type_definitions.push(TypeDefinition::Enum(enum_));
        Ok(())
    }

    fn process_interface(&self, py_ir: &mut PythonBindingsIr, int: ir::Interface) -> Result<()> {
        if int.has_callback_interface() {
            // This is a trait interface that can be implemented in Python, so it is treated like a
            // callback interface where the primary use-case is the trait being implemented
            // locally.  It is a base-class local implementations might subclass.
            // We reuse "Protocol.py" for this, even though here we are not generating a protocol
            py_ir.runtimes.callback_interface = true;
            if int.has_async_method() {
                py_ir.runtimes.async_callback = true;
            }
        }
        let (protocol, interface_name) = if int.has_callback_interface() {
            // This is a trait interface that can be implemented in Python, so it is treated like a
            // callback interface where the primary use-case is the trait being implemented
            // locally.  It is a base-class local implementations might subclass.
            // We reuse "Protocol.py" for this, even though here we are not generating a protocol
            let protocol = Protocol {
                name: self.class_name(&int.name),
                base_class: "".to_string(),
                docstring: self.format_docstring(int.docstring.clone()),
                methods: self.convert_methods(py_ir, int.methods.clone())?,
            };
            (protocol, format!("{}Impl", self.class_name(&int.name)))
        } else {
            let protocol = Protocol {
                name: format!("{}Protocol", self.class_name(&int.name)),
                base_class: "typing.Protocol".to_string(),
                docstring: self.format_docstring(int.docstring.clone()),
                methods: self.convert_methods(py_ir, int.methods.clone())?,
            };
            (protocol, self.class_name(&int.name))
        };
        let had_async_constructor = int.constructors.iter().any(|c| c.primary && c.is_async());

        let int = Interface {
            name: interface_name,
            protocol_name: protocol.name.clone(),
            constructors: self
                .convert_constructors(py_ir, int.constructors)?
                .into_iter()
                .map(|cons| {
                    // Python constructors can't be async.  If the primary constructor from Rust is async, then
                    // treat it like a secondary constructor which generates a factory method.
                    if cons.primary && cons.is_async() {
                        Constructor {
                            name: "new".to_string(),
                            primary: false,
                            ..cons
                        }
                    } else {
                        cons
                    }
                })
                .collect(),
            methods: self.convert_methods(py_ir, int.methods)?,
            had_async_constructor,
            uniffi_traits: self.convert_uniffi_traits(int.uniffi_traits)?,
            base_classes: std::iter::once(protocol.name.clone())
                .chain(
                    int.trait_impls
                        .iter()
                        .map(|trait_impl| self.class_name(&trait_impl.trait_name)),
                )
                .chain(
                    int.self_type
                        .is_used_as_error
                        .then(|| "Exception".to_string()),
                )
                .collect(),
            vtable: self.convert_opt_vtable(int.vtable)?,
            ffi_clone: int.ffi_clone.name,
            ffi_free: int.ffi_free.name,
            docstring: self.format_docstring(int.docstring),
            self_type: self.convert_type(int.self_type),
        };
        py_ir.exports.push(protocol.name.clone());
        py_ir.exports.push(int.name.clone());
        py_ir.protocols.push(protocol);
        py_ir.type_definitions.push(TypeDefinition::Interface(int));
        Ok(())
    }

    fn process_callback_interface(
        &self,
        py_ir: &mut PythonBindingsIr,
        cbi: ir::CallbackInterface,
    ) -> Result<()> {
        py_ir.runtimes.callback_interface = true;
        if cbi.has_async_method() {
            py_ir.runtimes.async_callback = true;
        }
        let cbi = CallbackInterface {
            name: self.class_name(&cbi.name),
            methods: self.convert_methods(py_ir, cbi.methods)?,
            vtable: self.convert_vtable(cbi.vtable)?,
            docstring: self.format_docstring(cbi.docstring),
            self_type: self.convert_type(cbi.self_type),
        };

        let protocol = Protocol {
            name: self.class_name(&cbi.name),
            base_class: "typing.Protocol".to_string(),
            docstring: cbi.docstring.clone(),
            methods: cbi.methods.clone(),
        };
        py_ir.exports.push(cbi.name.clone());
        py_ir.exports.push(protocol.name.clone());
        py_ir
            .type_definitions
            .push(TypeDefinition::CallbackInterface(cbi));
        py_ir.protocols.push(protocol);
        Ok(())
    }

    fn process_custom_type(
        &self,
        py_ir: &mut PythonBindingsIr,
        custom: ir::CustomType,
    ) -> Result<()> {
        let config = self.config.custom_types.get(&custom.name).cloned();
        if let Some(config) = &config {
            for mod_name in config.imports.iter().flatten() {
                py_ir.imports.insert(format!("import {mod_name}"));
            }
        }
        let custom = CustomType {
            config,
            name: self.class_name(&custom.name),
            builtin: self.convert_type(custom.builtin),
            self_type: self.convert_type(custom.self_type),
        };
        py_ir.exports.push(custom.name.clone());
        py_ir.type_definitions.push(TypeDefinition::Custom(custom));
        Ok(())
    }

    fn process_external_type(
        &self,
        py_ir: &mut PythonBindingsIr,
        ext: ir::ExternalType,
    ) -> Result<()> {
        let mod_name = self.config.module_for_namespace(&ext.namespace);
        let name = self.class_name(&ext.name);
        py_ir.imports.insert(format!("import {mod_name}"));
        py_ir
            .imports
            .insert(format!("from {mod_name} import {name}"));
        let ext = ExternalType {
            name,
            namespace: ext.namespace,
            kind: ext.kind,
            self_type: self.convert_type(ext.self_type),
        };
        py_ir.type_definitions.push(TypeDefinition::External(ext));
        Ok(())
    }

    fn process_function(&self, py_ir: &mut PythonBindingsIr, func: ir::Function) -> Result<()> {
        let func = Function {
            name: self.fn_name(&func.name),
            callable: self.convert_callable(func.callable)?,
            docstring: self.format_docstring(func.docstring),
        };
        if func.is_async() {
            self.process_async_func(py_ir);
        }
        py_ir.exports.push(func.name.clone());
        py_ir.functions.push(func);
        Ok(())
    }

    fn convert_callable(&self, callable: ir::Callable) -> Result<Callable> {
        Ok(Callable {
            kind: callable.kind,
            async_data: self.convert_async_data(callable.async_data),
            arguments: self.convert_arguments(callable.arguments)?,
            return_type: self.convert_opt_type(callable.return_type),
            throws_type: self.convert_opt_type(callable.throws_type),
            ffi_func: callable.ffi_func.name,
        })
    }

    fn convert_methods(
        &self,
        py_ir: &mut PythonBindingsIr,
        methods: Vec<ir::Method>,
    ) -> Result<Vec<Method>> {
        let methods = methods
            .into_iter()
            .map(|meth| self.convert_method(meth))
            .collect::<Result<Vec<_>>>()?;
        for meth in methods.iter() {
            if meth.is_async() {
                self.process_async_func(py_ir);
            }
        }
        Ok(methods)
    }

    fn convert_method(&self, meth: ir::Method) -> Result<Method> {
        Ok(Method {
            name: self.fn_name(&meth.name),
            interface: self.convert_type(meth.interface),
            callable: self.convert_callable(meth.callable)?,
            docstring: self.format_docstring(meth.docstring),
        })
    }

    fn convert_constructors(
        &self,
        py_ir: &mut PythonBindingsIr,
        constructors: Vec<ir::Constructor>,
    ) -> Result<Vec<Constructor>> {
        let constructors = constructors
            .into_iter()
            .map(|mut cons| {
                // Python constructors can't be async.  If the primary constructor from Rust is async, then
                // treat it like a secondary constructor which generates a factory method.
                let primary = cons.primary && !cons.is_async();
                // Make sure to update the CallableKind as well
                cons.callable.kind = ir::CallableKind::Constructor { primary };
                Ok(Constructor {
                    // Python constructors can't be async.  If the primary constructor from Rust is async, then
                    // treat it like a secondary constructor which generates a factory method.
                    name: if primary {
                        "__init__".to_string()
                    } else {
                        self.fn_name(&cons.name)
                    },
                    primary,
                    interface: self.convert_type(cons.interface),
                    callable: self.convert_callable(cons.callable)?,
                    docstring: self.format_docstring(cons.docstring),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        for cons in &constructors {
            if cons.is_async() {
                self.process_async_func(py_ir);
            }
        }
        Ok(constructors)
    }

    fn convert_opt_vtable(&self, vtable: Option<ir::VTable>) -> Result<Option<VTable>> {
        vtable.map(|vtable| self.convert_vtable(vtable)).transpose()
    }

    fn convert_vtable(&self, vtable: ir::VTable) -> Result<VTable> {
        Ok(VTable {
            name: self.ffi_struct_name(&vtable.name),
            ffi_type: self.ffi_type_name(vtable.ffi_type),
            ffi_init_callback: vtable.ffi_init_callback.name,
            methods: self.convert_vtable_methods(vtable.methods)?,
        })
    }

    fn convert_vtable_methods(&self, methods: Vec<ir::VTableMethod>) -> Result<Vec<VTableMethod>> {
        methods
            .into_iter()
            .map(|vmeth| {
                let default_return_value = match &vmeth.callable.return_type {
                    Some(t) => match &t.ffi_type {
                        ir::FfiType::UInt8
                        | ir::FfiType::Int8
                        | ir::FfiType::UInt16
                        | ir::FfiType::Int16
                        | ir::FfiType::UInt32
                        | ir::FfiType::Int32
                        | ir::FfiType::UInt64
                        | ir::FfiType::Int64
                        | ir::FfiType::Handle => "0".to_string(),
                        ir::FfiType::Float32 | ir::FfiType::Float64 => "0.0".to_string(),
                        ir::FfiType::RustArcPtr(_) => "ctypes.c_void_p()".to_string(),
                        ir::FfiType::RustBuffer(meta) => match meta {
                            None => "_UniffiRustBuffer.default()".to_string(),
                            Some(meta) => {
                                let module_name = self.config.module_for_namespace(&meta.namespace);
                                format!("{module_name}._UniffiRustBuffer.default()")
                            }
                        },
                        _ => unimplemented!("FFI default for: {t:?}"),
                    },
                    // When we need to use a value for void returns, we use a `u8` placeholder and `0` as
                    // the default.
                    None => "0".to_string(),
                };
                Ok(VTableMethod {
                    ffi_type: self.ffi_type_name(vmeth.ffi_type),
                    name: self.fn_name(&vmeth.name),
                    default_return_value,
                    callable: self.convert_callable(vmeth.callable)?,
                })
            })
            .collect()
    }

    fn convert_uniffi_traits(
        &self,
        uniffi_traits: Vec<ir::UniffiTrait>,
    ) -> Result<Vec<UniffiTrait>> {
        uniffi_traits
            .into_iter()
            .map(|ut| {
                Ok(match ut {
                    ir::UniffiTrait::Debug { fmt } => UniffiTrait::Debug {
                        fmt: Method {
                            name: "__repr__".to_string(),
                            ..self.convert_method(fmt)?
                        },
                    },
                    ir::UniffiTrait::Display { fmt } => UniffiTrait::Display {
                        fmt: Method {
                            name: "__str__".to_string(),
                            ..self.convert_method(fmt)?
                        },
                    },
                    ir::UniffiTrait::Eq { eq, ne } => UniffiTrait::Eq {
                        eq: Method {
                            name: "__eq__".to_string(),
                            ..self.convert_method(eq)?
                        },
                        ne: Method {
                            name: "__ne__".to_string(),
                            ..self.convert_method(ne)?
                        },
                    },
                    ir::UniffiTrait::Hash { hash } => UniffiTrait::Hash {
                        hash: Method {
                            name: "__hash__".to_string(),
                            ..self.convert_method(hash)?
                        },
                    },
                })
            })
            .collect()
    }

    fn convert_variants(&self, variants: Vec<ir::Variant>) -> Result<Vec<Variant>> {
        variants
            .into_iter()
            .map(|variant| {
                Ok(Variant {
                    name: if variant.enum_shape.is_error() {
                        self.class_name(&variant.name)
                    } else {
                        self.enum_variant_name(&variant.name)
                    },
                    docstring: self.format_docstring(variant.docstring),
                    discr: self.literal(variant.discr)?,
                    enum_shape: variant.enum_shape,
                    fields: self.convert_fields(variant.fields)?,
                })
            })
            .collect()
    }

    fn convert_arguments(&self, arguments: Vec<ir::Argument>) -> Result<Vec<Argument>> {
        arguments
            .into_iter()
            .map(|arg| {
                Ok(Argument {
                    name: self.var_name(&arg.name),
                    ty: self.convert_type(arg.ty),
                    default: self.convert_default(arg.default)?,
                })
            })
            .collect()
    }

    fn convert_fields(&self, fields: Vec<ir::Field>) -> Result<Vec<Field>> {
        fields
            .into_iter()
            .map(|field| {
                Ok(Field {
                    name: self.var_name(&field.name),
                    docstring: self.format_docstring(field.docstring),
                    ty: self.convert_type(field.ty),
                    default: self.convert_default(field.default)?,
                })
            })
            .collect()
    }

    fn convert_async_data(&self, async_data: Option<ir::AsyncData>) -> Option<AsyncData> {
        async_data.map(|async_data| AsyncData {
            ffi_rust_future_poll: async_data.ffi_rust_future_poll.name,
            ffi_rust_future_complete: async_data.ffi_rust_future_complete.name,
            ffi_rust_future_free: async_data.ffi_rust_future_free.name,
            foreign_future_result_type: self.ffi_type_name(async_data.foreign_future_result_type),
        })
    }

    fn convert_opt_type(&self, ty: Option<ir::Type>) -> Option<Type> {
        ty.map(|ty| self.convert_type(ty))
    }

    fn convert_type(&self, ty: ir::Type) -> Type {
        let ffi_converter_name = match &ty.kind {
            uniffi_meta::Type::External { namespace, .. } => {
                let mod_name = self.config.module_for_namespace(namespace);
                format!("{mod_name}._UniffiConverter{}", ty.canonical_name())
            }
            _ => format!("_UniffiConverter{}", ty.canonical_name()),
        };
        Type {
            type_name: self.type_name(&ty.kind),
            ffi_converter_name,
            kind: ty.kind,
            ffi_type: self.ffi_type_name(ty.ffi_type),
            is_used_as_error: ty.is_used_as_error,
        }
    }

    fn type_name(&self, ty: &uniffi_meta::Type) -> String {
        match ty {
            uniffi_meta::Type::Boolean => "bool".to_string(),
            uniffi_meta::Type::String => "str".to_string(),
            uniffi_meta::Type::Bytes => "bytes".to_string(),
            uniffi_meta::Type::Int8 => "int".to_string(),
            uniffi_meta::Type::Int16
            | uniffi_meta::Type::Int32
            | uniffi_meta::Type::Int64
            | uniffi_meta::Type::UInt8
            | uniffi_meta::Type::UInt16
            | uniffi_meta::Type::UInt32
            | uniffi_meta::Type::UInt64 => "int".to_string(),
            uniffi_meta::Type::Duration => "Duration".to_string(),
            uniffi_meta::Type::Timestamp => "Timestamp".to_string(),
            uniffi_meta::Type::Float32 | uniffi_meta::Type::Float64 => "float".to_string(),
            uniffi_meta::Type::Object { name, .. }
            | uniffi_meta::Type::Record { name, .. }
            | uniffi_meta::Type::Enum { name, .. }
            | uniffi_meta::Type::CallbackInterface { name, .. }
            | uniffi_meta::Type::Custom { name, .. }
            | uniffi_meta::Type::External { name, .. } => self.class_name(name),
            uniffi_meta::Type::Optional { inner_type } => {
                format!("typing.Optional[{}]", self.type_name(inner_type))
            }
            uniffi_meta::Type::Sequence { inner_type } => {
                format!("typing.List[{}]", self.type_name(inner_type))
            }
            uniffi_meta::Type::Map {
                key_type,
                value_type,
            } => format!(
                "dict[{}, {}]",
                self.type_name(key_type),
                self.type_name(value_type)
            ),
        }
    }

    fn process_async_func(&self, py_ir: &mut PythonBindingsIr) {
        py_ir.imports.insert("import asyncio".to_string());
        py_ir.runtimes.async_ = true;
    }

    /// Python type name for an FfiType
    fn ffi_type_name(&self, ffi_type: ir::FfiType) -> String {
        match ffi_type {
            ir::FfiType::Int8 => "ctypes.c_int8".to_string(),
            ir::FfiType::UInt8 => "ctypes.c_uint8".to_string(),
            ir::FfiType::Int16 => "ctypes.c_int16".to_string(),
            ir::FfiType::UInt16 => "ctypes.c_uint16".to_string(),
            ir::FfiType::Int32 => "ctypes.c_int32".to_string(),
            ir::FfiType::UInt32 => "ctypes.c_uint32".to_string(),
            ir::FfiType::Int64 => "ctypes.c_int64".to_string(),
            ir::FfiType::UInt64 => "ctypes.c_uint64".to_string(),
            ir::FfiType::Float32 => "ctypes.c_float".to_string(),
            ir::FfiType::Float64 => "ctypes.c_double".to_string(),
            ir::FfiType::Handle => "ctypes.c_uint64".to_string(),
            ir::FfiType::RustArcPtr(_) => "ctypes.c_void_p".to_string(),
            ir::FfiType::RustBuffer(meta) => match meta {
                None => "_UniffiRustBuffer".to_string(),
                Some(meta) => {
                    let module_name = self.config.module_for_namespace(&meta.namespace);
                    format!("{module_name}._UniffiRustBuffer")
                }
            },
            ir::FfiType::RustCallStatus => "_UniffiRustCallStatus".to_string(),
            ir::FfiType::ForeignBytes => "_UniffiForeignBytes".to_string(),
            ir::FfiType::FunctionPointer(name) => self.ffi_function_type_name(&name),
            ir::FfiType::Struct(name) => self.ffi_struct_name(&name),
            // Pointer to an `asyncio.EventLoop` instance
            ir::FfiType::Reference(inner) | ir::FfiType::MutReference(inner) => {
                format!("ctypes.POINTER({})", self.ffi_type_name(*inner))
            }
            ir::FfiType::VoidPointer => "ctypes.c_void_p".to_string(),
        }
    }

    /// Idiomatic name for an FFI function type.
    ///
    /// These follow the ctypes convention of SHOUTY_SNAKE_CASE. Prefix with `_UNIFFI` so that
    /// it can't conflict with user-defined items.
    fn ffi_function_type_name(&self, nm: &str) -> String {
        format!("_UNIFFI_FN_{}", nm.to_shouty_snake_case())
    }

    /// Idiomatic name for an FFI struct.
    ///
    /// These follow the ctypes convention of UpperCamelCase (although the ctypes docs also uses
    /// SHOUTY_SNAKE_CASE in some places).  Prefix with `_Uniffi` so that it can't conflict with
    /// user-defined items.
    fn ffi_struct_name(&self, nm: &str) -> String {
        format!("_UniffiStruct{}", nm.to_upper_camel_case())
    }

    /// Idiomatic Python class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &str) -> String {
        self.fixup_keyword(nm.to_string().to_upper_camel_case())
    }

    /// Idiomatic Python function name.
    fn fn_name(&self, nm: &str) -> String {
        self.fixup_keyword(nm.to_string().to_snake_case())
    }

    /// Idiomatic Python a variable name.
    fn var_name(&self, nm: &str) -> String {
        self.fixup_keyword(nm.to_string().to_snake_case())
    }

    /// Idiomatic Python enum variant name.
    ///
    /// These use SHOUTY_SNAKE_CASE style.  User's access them through `EnumName.VARIANT_NAME`.
    /// This naming style is used for both flat enums and fielded ones.
    ///
    /// The exception is error enums.  In that case, there's no enum in the generated python.
    /// Instead there's a base class and subclasses -- all with UpperCamelCase names.
    /// That case is not handled by this method.
    fn enum_variant_name(&self, nm: &str) -> String {
        self.fixup_keyword(nm.to_string().to_shouty_snake_case())
    }

    /// Fixup a name by ensuring it's not a keyword
    fn fixup_keyword(&self, name: String) -> String {
        if KEYWORDS.contains(&name) {
            format!("_{name}")
        } else {
            name
        }
    }

    pub fn format_docstring(&self, docstring: Option<String>) -> Option<String> {
        docstring.map(|docstring| {
            // Escape triple quotes to avoid syntax errors in docstrings
            let docstring = docstring.replace(r#"""""#, r#"\"\"\""#);
            // Remove indentation and surround with quotes
            format!("\"\"\"\n{}\n\"\"\"", &textwrap::dedent(&docstring))
        })
    }

    fn convert_default(&self, default: Option<ir::Literal>) -> Result<Option<String>> {
        default.map(|lit| self.literal(lit)).transpose()
    }

    /// Python rendering of a literal value
    fn literal(&self, lit: ir::Literal) -> Result<String> {
        Ok(match lit {
            ir::Literal::Boolean(true) => "True".to_string(),
            ir::Literal::Boolean(false) => "False".to_string(),
            ir::Literal::String(s) => format!("\"{s}\""),
            // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
            ir::Literal::Int(i, radix, _) => match radix {
                ir::Radix::Octal => format!("int(0o{i:o})"),
                ir::Radix::Decimal => format!("{i}"),
                ir::Radix::Hexadecimal => format!("{i:#x}"),
            },
            ir::Literal::UInt(i, radix, _) => match radix {
                ir::Radix::Octal => format!("0o{i:o}"),
                ir::Radix::Decimal => format!("{i}"),
                ir::Radix::Hexadecimal => format!("{i:#x}"),
            },
            ir::Literal::Float(value, _) => value.clone(),
            ir::Literal::EmptySequence => "[]".to_string(),
            ir::Literal::EmptyMap => "{}".to_string(),
            ir::Literal::None => "None".to_string(),
            ir::Literal::Some { inner } => self.literal(*inner)?,
            ir::Literal::Enum(variant, ty) => match &ty.kind {
                uniffi_meta::Type::Enum { name, .. } => {
                    format!(
                        "{}.{}",
                        self.class_name(name),
                        self.enum_variant_name(&variant)
                    )
                }
                type_kind => {
                    bail!("Invalid type for enum literal: {type_kind:?}")
                }
            },
        })
    }
}

// Taken from Python's `keyword.py` module.
static KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    let kwlist = vec![
        "False",
        "None",
        "True",
        "__peg_parser__",
        "and",
        "as",
        "assert",
        "async",
        "await",
        "break",
        "class",
        "continue",
        "def",
        "del",
        "elif",
        "else",
        "except",
        "finally",
        "for",
        "from",
        "global",
        "if",
        "import",
        "in",
        "is",
        "lambda",
        "nonlocal",
        "not",
        "or",
        "pass",
        "raise",
        "return",
        "try",
        "while",
        "with",
        "yield",
    ];
    HashSet::from_iter(kwlist.into_iter().map(|s| s.to_string()))
});
