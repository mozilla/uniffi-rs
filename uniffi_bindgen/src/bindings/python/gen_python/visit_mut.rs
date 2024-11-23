/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use once_cell::sync::Lazy;

use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

use super::Config;
use crate::interface::ir::*;

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

/// Implements `VisitMut` to specialize the BindingsIr for Python
pub struct BindingsIrVisitor {
    pub config: Config,
    // The following fields are populated as we walk the tree for VisitMut
    pub imports: BTreeSet<String>,
    pub protocols: Vec<Protocol>,
    pub runtimes: Runtimes,
    pub exports: Vec<String>,
}

// Runtimes to generate
//
// These are sections of helper code that we load once
#[derive(Default, Debug)]
pub struct Runtimes {
    pub async_: bool,
    pub async_callback: bool,
    pub callback_interface: bool,
}

/// Protocol to define
pub struct Protocol {
    pub name: String,
    pub base_class: String,
    pub docstring: Option<String>,
    pub methods: Vec<Method>,
}

impl BindingsIrVisitor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            imports: BTreeSet::new(),
            runtimes: Runtimes::default(),
            protocols: vec![],
            exports: vec![],
        }
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

    /// Python type name
    fn type_name(&self, ty: &Type) -> String {
        match &ty.kind {
            TypeKind::Boolean => "bool".to_string(),
            TypeKind::String => "str".to_string(),
            TypeKind::Bytes => "bytes".to_string(),
            TypeKind::Int8 => "int".to_string(),
            TypeKind::Int16
            | TypeKind::Int32
            | TypeKind::Int64
            | TypeKind::UInt8
            | TypeKind::UInt16
            | TypeKind::UInt32
            | TypeKind::UInt64 => "int".to_string(),
            TypeKind::Duration => "Duration".to_string(),
            TypeKind::Timestamp => "Timestamp".to_string(),
            TypeKind::Float32 | TypeKind::Float64 => "float".to_string(),
            TypeKind::Interface { name, .. }
            | TypeKind::Record { name, .. }
            | TypeKind::Enum { name, .. }
            | TypeKind::CallbackInterface { name, .. }
            | TypeKind::Custom { name, .. }
            | TypeKind::External { name, .. } => self.class_name(name),
            TypeKind::Optional { inner_type } => {
                format!("typing.Optional[{}]", self.type_name(inner_type))
            }
            TypeKind::Sequence { inner_type } => {
                format!("typing.List[{}]", self.type_name(inner_type))
            }
            TypeKind::Map {
                key_type,
                value_type,
            } => format!(
                "dict[{}, {}]",
                self.type_name(key_type),
                self.type_name(value_type)
            ),
        }
    }

    /// Python rendering of a literal value
    fn literal(&self, lit: &Literal) -> Result<String> {
        Ok(match &lit.kind {
            LiteralKind::Boolean(true) => "True".to_string(),
            LiteralKind::Boolean(false) => "False".to_string(),
            LiteralKind::String(s) => format!("\"{s}\""),
            // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
            LiteralKind::Int(i, radix, _) => match radix {
                Radix::Octal => format!("int(0o{i:o})"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
            LiteralKind::UInt(i, radix, _) => match radix {
                Radix::Octal => format!("0o{i:o}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
            LiteralKind::Float(value, _) => value.clone(),
            LiteralKind::EmptySequence => "[]".to_string(),
            LiteralKind::EmptyMap => "{}".to_string(),
            LiteralKind::None => "None".to_string(),
            LiteralKind::Some { inner } => self.literal(inner)?,
            LiteralKind::Enum(variant, ty) => match &ty.kind {
                TypeKind::Enum { name, .. } => {
                    format!(
                        "{}.{}",
                        self.class_name(name),
                        self.enum_variant_name(variant)
                    )
                }
                _ => {
                    bail!("Invalid type for enum literal: {ty:?} ({lit:?})")
                }
            },
        })
    }

    /// Python type name for an FfiType
    fn ffi_type_name(&self, ffi_type: &FfiType) -> String {
        match &ffi_type.kind {
            FfiTypeKind::Int8 => "ctypes.c_int8".to_string(),
            FfiTypeKind::UInt8 => "ctypes.c_uint8".to_string(),
            FfiTypeKind::Int16 => "ctypes.c_int16".to_string(),
            FfiTypeKind::UInt16 => "ctypes.c_uint16".to_string(),
            FfiTypeKind::Int32 => "ctypes.c_int32".to_string(),
            FfiTypeKind::UInt32 => "ctypes.c_uint32".to_string(),
            FfiTypeKind::Int64 => "ctypes.c_int64".to_string(),
            FfiTypeKind::UInt64 => "ctypes.c_uint64".to_string(),
            FfiTypeKind::Float32 => "ctypes.c_float".to_string(),
            FfiTypeKind::Float64 => "ctypes.c_double".to_string(),
            FfiTypeKind::Handle => "ctypes.c_uint64".to_string(),
            FfiTypeKind::RustArcPtr(_) => "ctypes.c_void_p".to_string(),
            FfiTypeKind::RustBuffer(meta) => match meta {
                None => "_UniffiRustBuffer".to_string(),
                Some(meta) => {
                    let module_name = self.config.module_for_namespace(&meta.namespace);
                    format!("{module_name}._UniffiRustBuffer")
                }
            },
            FfiTypeKind::RustCallStatus => "_UniffiRustCallStatus".to_string(),
            FfiTypeKind::ForeignBytes => "_UniffiForeignBytes".to_string(),
            FfiTypeKind::FunctionPointer(name) => self.ffi_function_type_name(name),
            FfiTypeKind::Struct(name) => self.ffi_struct_name(name),
            // Pointer to an `asyncio.EventLoop` instance
            FfiTypeKind::Reference(inner) | FfiTypeKind::MutReference(inner) => {
                format!("ctypes.POINTER({})", self.ffi_type_name(inner))
            }
            FfiTypeKind::VoidPointer => "ctypes.c_void_p".to_string(),
        }
    }

    /// Idiomatic name for an FFI function type.
    ///
    /// These follow the ctypes convention of SHOUTY_SNAKE_CASE. Prefix with `_UNIFFI` so that
    /// it can't conflict with user-defined items.
    fn ffi_function_type_name(&self, nm: &str) -> String {
        format!("_UNIFFI_{}", nm.to_shouty_snake_case())
    }

    /// Idiomatic name for an FFI struct.
    ///
    /// These follow the ctypes convention of UpperCamelCase (although the ctypes docs also uses
    /// SHOUTY_SNAKE_CASE in some places).  Prefix with `_Uniffi` so that it can't conflict with
    /// user-defined items.
    fn ffi_struct_name(&self, nm: &str) -> String {
        format!("_Uniffi{}", nm.to_upper_camel_case())
    }

    /// Push to `self.imports` based on a type we've observed in the IR.
    fn add_imports_for_type(&mut self, ty: &Type) {
        match &ty.kind {
            TypeKind::Custom { name, .. } => {
                if let Some(config) = self.config.custom_types.get(name.as_str()) {
                    for mod_name in config.imports.iter().flatten() {
                        self.imports.insert(format!("import {mod_name}"));
                    }
                }
            }
            TypeKind::External {
                name, namespace, ..
            } => {
                let mod_name = self.config.module_for_namespace(namespace);
                let name = self.class_name(name);
                self.imports.insert(format!("import {mod_name}"));
                self.imports
                    .insert(format!("from {mod_name} import {name}"));
            }
            _ => (),
        }
    }

    fn add_module_import(&mut self, module_name: &str) {
        self.imports.insert(format!("import {module_name}"));
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
}

impl VisitMut for BindingsIrVisitor {
    fn visit_bindings_ir(&mut self, bindings_ir: &mut BindingsIr) -> Result<()> {
        bindings_ir.crate_docstring = self.format_docstring(bindings_ir.crate_docstring.take());
        Ok(())
    }

    fn visit_record(&mut self, rec: &mut Record) -> Result<()> {
        rec.name = self.class_name(&rec.name);
        rec.docstring = self.format_docstring(rec.docstring.take());
        self.exports.push(rec.name.clone());
        Ok(())
    }

    fn visit_interface(&mut self, interface: &mut Interface) -> Result<()> {
        // Make sure to setup docstring before we clone it for our Protocol.
        interface.docstring = self.format_docstring(interface.docstring.take());
        if interface.has_callback_interface() {
            // This is a trait interface that can be implemented in Python, so it is treated like a
            // callback interface where the primary use-case is the trait being implemented
            // locally.  It is a base-class local implementations might subclass.
            // We reuse "Protocol.py" for this, even though here we are not generating a protocol
            self.runtimes.callback_interface = true;
            if interface.has_async_method() {
                self.runtimes.async_callback = true;
            }
            let protocol = Protocol {
                name: self.class_name(&interface.name),
                base_class: "".to_string(),
                docstring: interface.docstring.clone(),
                methods: interface.methods.clone(),
            };
            interface.name = format!("{}Impl", self.class_name(&interface.name));
            interface
                .lang_data
                .insert("protocol_name", protocol.name.clone());
            self.exports.push(protocol.name.clone());
            self.protocols.push(protocol);
        } else {
            let protocol = Protocol {
                name: format!("{}Protocol", self.class_name(&interface.name)),
                base_class: "typing.Protocol".to_string(),
                docstring: interface.docstring.clone(),
                methods: interface.methods.clone(),
            };
            interface.name = self.class_name(&interface.name);
            interface
                .lang_data
                .insert("protocol_name", protocol.name.clone());
            self.exports.push(interface.name.clone());
            self.exports.push(protocol.name.clone());
            self.protocols.push(protocol);
        }
        for i in interface.trait_impls.iter_mut() {
            i.trait_name = self.class_name(&i.trait_name);
        }
        // Python constructors can't be async.  If the primary constructor from Rust is async, then
        // treat it like a secondary constructor which generates a factory method.
        if let Some(cons) = interface
            .constructors
            .iter_mut()
            .find(|c| c.primary && c.is_async())
        {
            cons.name = "new".to_string();
            cons.primary = false;
            interface.lang_data.insert("had_async_constructor", true);
        }

        interface.lang_data.insert(
            "base_classes",
            interface
                .trait_impls
                .iter()
                .map(|trait_impl| trait_impl.trait_name.as_str())
                .chain(interface.is_used_as_error().then_some("Exception"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        Ok(())
    }

    fn visit_callback_interface(&mut self, cbi: &mut CallbackInterface) -> Result<()> {
        self.runtimes.callback_interface = true;
        cbi.name = self.class_name(&cbi.name);
        cbi.docstring = self.format_docstring(cbi.docstring.take());
        if cbi.has_async_method() {
            self.runtimes.async_callback = true;
        }
        self.protocols.push(Protocol {
            name: self.class_name(&cbi.name),
            base_class: "typing.Protocol".to_string(),
            docstring: cbi.docstring.clone(),
            methods: cbi.methods.clone(),
        });
        self.exports.push(cbi.name.clone());
        Ok(())
    }

    fn visit_vtable(&mut self, vtable: &mut VTable) -> Result<()> {
        vtable.name = format!("_UniffiVTable{}", self.class_name(&vtable.name));
        Ok(())
    }

    fn visit_field(&mut self, field: &mut Field) -> Result<()> {
        field.name = self.var_name(&field.name);
        field.docstring = self.format_docstring(field.docstring.take());
        Ok(())
    }

    fn visit_enum(&mut self, enum_: &mut Enum) -> Result<()> {
        enum_.name = self.class_name(&enum_.name);
        enum_.docstring = self.format_docstring(enum_.docstring.take());
        self.exports.push(enum_.name.clone());
        Ok(())
    }

    fn visit_custom_type(&mut self, custom: &mut CustomType) -> Result<()> {
        if let Some(config) = self.config.custom_types.get(&custom.name) {
            custom.lang_data.insert("custom_type_config", config);
        }
        custom.name = self.class_name(&custom.name);
        self.exports.push(custom.name.clone());
        Ok(())
    }

    fn visit_external_type(&mut self, ext: &mut ExternalType) -> Result<()> {
        ext.name = self.class_name(&ext.name);
        Ok(())
    }

    fn visit_variant(&mut self, variant: &mut Variant) -> Result<()> {
        if variant.enum_shape.is_error() {
            variant.name = self.class_name(&variant.name);
        } else {
            variant.name = self.enum_variant_name(&variant.name);
        }
        variant.docstring = self.format_docstring(variant.docstring.take());
        Ok(())
    }

    fn visit_method(&mut self, meth: &mut Method) -> Result<()> {
        meth.name = self.fn_name(&meth.name);
        meth.docstring = self.format_docstring(meth.docstring.take());
        if meth.is_async() {
            self.add_module_import("asyncio");
            self.runtimes.async_ = true;
        }
        Ok(())
    }

    fn visit_uniffi_trait(&mut self, ut: &mut UniffiTrait) -> Result<()> {
        match ut {
            UniffiTrait::Debug { fmt } => {
                fmt.name = "__repr__".to_string();
            }
            UniffiTrait::Display { fmt } => {
                fmt.name = "__str__".to_string();
            }
            UniffiTrait::Eq { eq, ne } => {
                eq.name = "__eq__".to_string();
                ne.name = "__ne__".to_string();
            }
            UniffiTrait::Hash { hash } => {
                hash.name = "__hash__".to_string();
            }
        }
        Ok(())
    }

    fn visit_argument(&mut self, arg: &mut Argument) -> Result<()> {
        arg.name = self.var_name(&arg.name);
        Ok(())
    }

    fn visit_constructor(&mut self, cons: &mut Constructor) -> Result<()> {
        cons.name = if cons.is_primary_constructor() && !cons.is_async() {
            "__init__".to_string()
        } else {
            self.fn_name(&cons.name)
        };
        cons.docstring = self.format_docstring(cons.docstring.take());
        if cons.is_async() {
            self.add_module_import("asyncio");
            self.runtimes.async_ = true;
        }
        Ok(())
    }

    fn visit_function(&mut self, func: &mut Function) -> Result<()> {
        func.name = self.fn_name(&func.name);
        func.docstring = self.format_docstring(func.docstring.take());
        if func.is_async() {
            self.add_module_import("asyncio");
            self.runtimes.async_ = true;
        }
        self.exports.push(func.name.clone());
        Ok(())
    }

    fn visit_type(&mut self, ty: &mut Type) -> Result<()> {
        self.add_imports_for_type(ty);
        let ffi_converter_name = format!("_UniffiConverter{}", ty.canonical_name());
        match &ty.kind {
            TypeKind::External { namespace, .. } => {
                let mod_name = self.config.module_for_namespace(namespace);
                ty.lang_data.insert(
                    "ffi_converter_name",
                    format!("{mod_name}.{ffi_converter_name}"),
                );
            }
            _ => ty
                .lang_data
                .insert("ffi_converter_name", ffi_converter_name),
        }
        ty.lang_data.insert("type_name", self.type_name(ty));
        Ok(())
    }

    fn visit_return_type(&mut self, return_type: &mut ReturnType) -> Result<()> {
        let ffi_default = match &return_type.ty {
            Some(t) => match &t.ffi_type.kind {
                FfiTypeKind::UInt8
                | FfiTypeKind::Int8
                | FfiTypeKind::UInt16
                | FfiTypeKind::Int16
                | FfiTypeKind::UInt32
                | FfiTypeKind::Int32
                | FfiTypeKind::UInt64
                | FfiTypeKind::Int64
                | FfiTypeKind::Handle => "0".to_string(),
                FfiTypeKind::Float32 | FfiTypeKind::Float64 => "0.0".to_string(),
                FfiTypeKind::RustArcPtr(_) => "ctypes.c_void_p()".to_string(),
                FfiTypeKind::RustBuffer(meta) => match meta {
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
        return_type.lang_data.insert("ffi_default", ffi_default);
        Ok(())
    }

    fn visit_literal(&mut self, literal: &mut Literal) -> Result<()> {
        literal
            .lang_data
            .insert("rendered_literal", self.literal(literal)?);
        Ok(())
    }

    fn visit_ffi_function_type(&mut self, func_type: &mut FfiFunctionType) -> Result<()> {
        func_type.name = self.ffi_function_type_name(&func_type.name);
        Ok(())
    }

    fn visit_ffi_struct(&mut self, struct_: &mut FfiStruct) -> Result<()> {
        struct_.name = self.ffi_struct_name(&struct_.name);
        Ok(())
    }

    fn visit_ffi_type(&mut self, node: &mut FfiType) -> Result<()> {
        node.lang_data
            .insert("ffi_type_name", self.ffi_type_name(node));
        Ok(())
    }
}
