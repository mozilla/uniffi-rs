/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::*;
/// Toplevel AST items
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::iter::IntoIterator;

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum Definition {
    BufferStream(BufferStreamDef),
    Function(FunctionDef),
    CStruct(CStructDef),
    Class(ClassDef),
    DataClass(DataClassDef),
    Enum(EnumDef),
    ExceptionBase(ExceptionBaseDef),
    Exception(ExceptionDef),
}

impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Self::BufferStream(buf) => &buf.name,
            Self::Function(func) => &func.name,
            Self::CStruct(cstruct) => &cstruct.name,
            Self::Class(class) => &class.name,
            Self::DataClass(data_class) => &data_class.name,
            Self::Enum(enum_) => &enum_.name,
            Self::ExceptionBase(base) => &base.name,
            Self::Exception(exc) => &exc.name,
        }
    }

    pub fn as_buffer(&self) -> Option<&BufferStreamDef> {
        match self {
            Self::BufferStream(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_cstruct(&self) -> Option<&CStructDef> {
        match self {
            Self::CStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&FunctionDef> {
        match self {
            Self::Function(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_class(&self) -> Option<&ClassDef> {
        match self {
            Self::Class(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_data_class(&self) -> Option<&DataClassDef> {
        match self {
            Self::DataClass(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumDef> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_exception_base(&self) -> Option<&ExceptionBaseDef> {
        match self {
            Self::ExceptionBase(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_exception(&self) -> Option<&ExceptionDef> {
        match self {
            Self::Exception(inner) => Some(inner),
            _ => None,
        }
    }
}

/// Built-in object type that supports reading from/writing to a data buffer.
///
/// BufferStreams have an interal position that tracks where in the buffer the next read/write will
/// happen.  Use `BufStreamRead*` and `BufStreamWrite*` expressions to perform the
/// read/write.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct BufferStreamDef {
    /// Name of the buffer class, must match the value in `Type::Pointer { name }`
    pub name: ClassName,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct CStructDef {
    pub name: CStructName,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct FunctionDef {
    pub vis: Visibility,
    pub name: FunctionName,
    /// Name of the exception class that this function may throw.  May be an Exception or
    /// ExceptionBase
    pub throws: Option<ClassName>,
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct ClassDef {
    pub vis: Visibility,
    pub name: ClassName,
    pub fields: Vec<Field>,
    pub constructor: Option<Constructor>,
    pub methods: Vec<Method>,
    pub destructor: Option<Destructor>,
    pub into_rust: Option<IntoRustMethod>,
}

/// Data classes simply store other objects as fields.  Languages often offer special support for
/// them, for example auto-deriving hash functions, equality tests etc.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct DataClassDef {
    pub vis: Visibility,
    pub name: ClassName,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct EnumDef {
    pub vis: Visibility,
    pub name: ClassName,
    pub variants: Vec<Variant>,
}

/// Base class for an exception
///
/// In general, bindings-ir to avoid inheritance since languages can treat it very
/// differently or not support it at all.  However, supporting exception hierarchies is very
/// useful.  To balance those 2 goals, we support base classes, but don't allow any fields or
/// methods on them.
///
/// Exceptions are the only type that supports variance.  If a function inputs an argument with the
/// exception base class, then any descendent classes can be passed in.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct ExceptionBaseDef {
    /// Depth of this base in the exception hierarchy.  Having this as the first field ensures that
    /// parent bases ordered before their children, which simplifies rendering.  This gets set when
    /// the exception base is added as a definition.
    pub(crate) depth: u32,
    /// Name of the parent ExceptionBase if there is one
    pub parent: Option<ClassName>,
    pub name: ClassName,
}

/// Exception class
///
/// Like ExceptionBase, exceptions are another OO concept that we need to support, but we can't
/// assume much about how languages will treat them.  So, we support defining and throwing
/// exceptions, but not catching them.
///
/// Exceptions always have public visibility.
#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct ExceptionDef {
    /// Name of the parent ExceptionBase
    pub parent: ClassName,
    /// Name of the Exception
    pub name: ClassName,
    pub fields: Vec<Field>,
    pub as_string: Option<AsStringMethod>,
}

#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize,
)]
#[serde(tag = "ir_type")]
pub enum Visibility {
    /// Visible to consumer code
    Public,
    /// Visible to the generated bindings only (note this means it's closer to Kotlin's `internal`
    /// than `private`
    #[default]
    Private,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Method {
    pub vis: Visibility,
    // Does this method mutate the object?
    pub method_type: MethodType,
    pub name: FunctionName,
    /// Name of the exception class that this function may throw.  May be an Exception or
    /// ExceptionBase
    pub throws: Option<ClassName>,
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum MethodType {
    #[default]
    Normal,
    Mutable,
    Static,
}

/// Main constructor for the class.
#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Constructor {
    pub vis: Visibility,
    pub args: Vec<Argument>,
    /// Initial value for for each class field
    pub initializers: Vec<Expression>,
}

/// Destructor which should free any Rust resources from this class.
///
/// All fields of the class will be available as owned variables in the body of the destructor.
/// Different languages handle destruction differently and there may be a long delay before
/// destructor calls. As a general rule, destructors should be used to free memory, but not
/// resources like file handles.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Destructor {
    pub body: Block,
}

/// Transfer ownership of class data into Rust
///
/// This function handles the `IntoRust` expression, which passes ownership of an object into Rust.
/// We can't directly a class directly, so this function must convert the object data into a
/// pointer, CStruct, or other FFI type that can be passed in. Like destructors, all fields of the
/// class will be available as owned variables in the body.
///
/// If the into rust method runs, then the object is considered not owned by the bindings code and
/// the destructor will not run.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct IntoRustMethod {
    pub return_type: Type,
    pub body: Block,
}

/// Get a string representation for an object
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct AsStringMethod {
    pub body: Block,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Argument {
    pub name: ArgName,
    #[serde(rename = "type")]
    pub type_: Type,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Field {
    pub name: FieldName,
    #[serde(rename = "type")]
    pub type_: Type,
    pub mutable: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Variant {
    pub name: ClassName,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct NativeLibrary {
    pub name: String,
    pub functions: BTreeMap<String, FFIFunctionDef>,
}

/// Function defined in an FFI Library
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct FFIFunctionDef {
    // Note that name is a bare string here, since FFI function names come from the C API
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
}

impl NativeLibrary {
    pub fn new(
        name: impl Into<String>,
        functions: impl IntoIterator<Item = FFIFunctionDef>,
    ) -> Self {
        Self {
            name: name.into(),
            functions: functions
                .into_iter()
                .map(|f| (f.name.to_string(), f))
                .collect(),
        }
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = &FFIFunctionDef> {
        self.functions.values()
    }
}

impl EnumDef {
    pub fn get_variant(&self, variant_name: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| v.name.equals(variant_name))
    }

    pub fn has_fields(&self) -> bool {
        self.variants.iter().any(Variant::has_fields)
    }
}

impl Variant {
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }
}

impl ExceptionBaseDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            depth: 0,
            name: ClassName::new(name.into()),
            parent: None,
        }
    }

    pub fn new_child(parent_name: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            depth: 0,
            name: ClassName::new(name.into()),
            parent: Some(ClassName::new(parent_name.into())),
        }
    }
}
