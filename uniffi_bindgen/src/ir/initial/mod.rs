/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Initial IR, this is essentially the Metadata from uniffi_meta without changes.

use std::fs;

use anyhow::Result;
use camino::Utf8Path;
use indexmap::IndexMap;

use crate::{
    crate_name_from_cargo_toml, interface,
    ir::{ir, FromNode, IntoNode, Node},
    macro_metadata, BindgenCrateConfigSupplier,
};

ir! {
    name: initial;

    /// Initial IR, this stores the metadata and other data
    #[derive(Debug, Clone, Default, Node, PartialEq)]
    pub struct Root {
        pub metadata: Vec<Metadata>,
        /// Map namespaces to docstrings -- this only works with UDL files.
        /// Eventually, this should just be another metadata item.
        pub docstrings: IndexMap<String, String>,
        /// In library mode, the library path the user passed to us
        pub cdylib: Option<String>,
    }

    // Metadata types, these are exact clones of the types from uniffi_meta, except the some type
    // names are changed.
    #[derive(Debug, Clone, Node, PartialEq)]
    #[from_uniffi_meta(Metadata)]
    pub enum Metadata {
        Namespace(NamespaceMetadata),
        UdlFile(UdlFile),
        Func(Function),
        #[from_uniffi_meta(Object)]
        Interface(Interface),
        CallbackInterface(CallbackInterface),
        Record(Record),
        Enum(Enum),
        Constructor(Constructor),
        Method(Method),
        TraitMethod(TraitMethod),
        CustomType(CustomType),
        UniffiTrait(UniffiTrait),
        ObjectTraitImpl(ObjectTraitImpl),
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(NamespaceMetadata)]
    pub struct NamespaceMetadata {
        pub crate_name: String,
        pub name: String,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(UdlFile)]
    pub struct UdlFile {
        // The module path specified when the UDL file was parsed.
        pub module_path: String,
        pub namespace: String,
        // the base filename of the udl file - no path, no extension.
        pub file_stub: String,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(FnMetadata)]
    pub struct Function {
        pub module_path: String,
        pub name: String,
        pub is_async: bool,
        pub inputs: Vec<Argument>,
        pub return_type: Option<Type>,
        pub throws: Option<Type>,
        pub checksum: Option<u16>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(ConstructorMetadata)]
    pub struct Constructor {
        pub module_path: String,
        pub self_name: String,
        pub name: String,
        pub is_async: bool,
        pub inputs: Vec<Argument>,
        pub throws: Option<Type>,
        pub checksum: Option<u16>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(MethodMetadata)]
    pub struct Method {
        pub module_path: String,
        pub self_name: String,
        pub name: String,
        pub is_async: bool,
        pub inputs: Vec<Argument>,
        pub return_type: Option<Type>,
        pub throws: Option<Type>,
        pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
        pub checksum: Option<u16>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(TraitMethodMetadata)]
    pub struct TraitMethod {
        pub module_path: String,
        pub trait_name: String,
        // Note: the position of `index` is important since it causes callback interface methods to be
        // ordered correctly in MetadataGroup.items
        pub index: u32,
        pub name: String,
        pub is_async: bool,
        pub inputs: Vec<Argument>,
        pub return_type: Option<Type>,
        pub throws: Option<Type>,
        pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
        pub checksum: Option<u16>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(FnParamMetadata)]
    pub struct Argument {
        pub name: String,
        pub ty: Type,
        pub by_ref: bool,
        pub optional: bool,
        pub default: Option<LiteralNode>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct LiteralNode {
        pub lit: Literal,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(LiteralMetadata)]
    pub enum Literal {
        Boolean(bool),
        String(String),
        // Integers are represented as the widest representation we can.
        // Number formatting vary with language and radix, so we avoid a lot of parsing and
        // formatting duplication by using only signed and unsigned variants.
        UInt(u64, Radix, Type),
        Int(i64, Radix, Type),
        // Pass the string representation through as typed in the UDL.
        // This avoids a lot of uncertainty around precision and accuracy,
        // though bindings for languages less sophisticated number parsing than WebIDL
        // will have to do extra work.
        Float(String, Type),
        Enum(String, Type),
        EmptySequence,
        EmptyMap,
        None,
        Some { inner: Box<Literal> },
    }

    // Represent the radix of integer literal values.
    // We preserve the radix into the generated bindings for readability reasons.
    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(Radix)]
    pub enum Radix {
        Decimal = 10,
        Octal = 8,
        Hexadecimal = 16,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(RecordMetadata)]
    pub struct Record {
        pub module_path: String,
        pub name: String,
        pub remote: bool, // only used when generating scaffolding from UDL
        pub fields: Vec<Field>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(FieldMetadata)]
    pub struct Field {
        pub name: String,
        pub ty: Type,
        pub default: Option<LiteralNode>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(EnumShape)]
    pub enum EnumShape {
        Enum,
        Error { flat: bool },
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(EnumMetadata)]
    pub struct Enum {
        pub module_path: String,
        pub name: String,
        pub shape: EnumShape,
        pub remote: bool,
        pub variants: Vec<Variant>,
        pub discr_type: Option<Type>,
        pub non_exhaustive: bool,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(VariantMetadata)]
    pub struct Variant {
        pub name: String,
        pub discr: Option<LiteralNode>,
        pub fields: Vec<Field>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(ObjectMetadata)]
    pub struct Interface {
        pub module_path: String,
        pub name: String,
        pub remote: bool, // only used when generating scaffolding from UDL
        pub imp: ObjectImpl,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(CallbackInterfaceMetadata)]
    pub struct CallbackInterface {
        pub module_path: String,
        pub name: String,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(UniffiTraitMetadata)]
    pub enum UniffiTrait {
        Debug {
            fmt: Method,
        },
        Display {
            fmt: Method,
        },
        Eq {
            eq: Method,
            ne: Method,
        },
        Hash {
            hash: Method,
        },
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(ObjectTraitImplMetadata)]
    pub struct ObjectTraitImpl {
        pub ty: Type,
        pub trait_name: String,
        pub tr_module_path: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    #[from_uniffi_meta(CustomTypeMetadata)]
    pub struct CustomType {
        pub module_path: String,
        pub name: String,
        pub builtin: Type,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Node)]
    #[from_uniffi_meta(Type)]
    pub enum Type {
        // Primitive types.
        UInt8,
        Int8,
        UInt16,
        Int16,
        UInt32,
        Int32,
        UInt64,
        Int64,
        Float32,
        Float64,
        Boolean,
        String,
        Bytes,
        Timestamp,
        Duration,
        #[from_uniffi_meta(Object)]
        Interface {
            // The module path to the object
            module_path: String,
            // The name in the "type universe"
            name: String,
            // How the object is implemented.
            imp: ObjectImpl,
        },
        // Types defined in the component API, each of which has a string name.
        Record {
            module_path: String,
            name: String,
        },
        Enum {
            module_path: String,
            name: String,
        },
        CallbackInterface {
            module_path: String,
            name: String,
        },
        // Structurally recursive types.
        Optional {
            inner_type: Box<Type>,
        },
        Sequence {
            inner_type: Box<Type>,
        },
        Map {
            key_type: Box<Type>,
            value_type: Box<Type>,
        },
        // Custom type on the scaffolding side
        Custom {
            module_path: String,
            name: String,
            builtin: Box<Type>,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Node)]
    #[from_uniffi_meta(ObjectImpl)]
    pub enum ObjectImpl {
        // A single Rust type
        Struct,
        // A trait that's can be implemented by Rust types
        Trait,
        // A trait + a callback interface -- can be implemented by both Rust and foreign types.
        CallbackTrait,
    }
}

impl FromNode<uniffi_meta::LiteralMetadata> for LiteralNode {
    fn from_node(lit: uniffi_meta::LiteralMetadata) -> Result<Self> {
        Ok(Self {
            lit: lit.into_node()?,
        })
    }
}

impl Root {
    pub fn from_library(
        config_supplier: impl BindgenCrateConfigSupplier,
        path: &Utf8Path,
        crate_name: Option<String>,
    ) -> Result<Root> {
        let mut all_metadata = macro_metadata::extract_from_library(path)?;
        if let Some(crate_name) = crate_name {
            all_metadata.retain(|meta| meta.module_path().split("::").next() == Some(&crate_name));
        }

        let mut metadata = vec![];
        let mut udl_to_load = vec![];

        for meta in macro_metadata::extract_from_library(path)? {
            match meta {
                uniffi_meta::Metadata::UdlFile(udl) => {
                    udl_to_load.push((
                        config_supplier.get_udl(&udl.module_path, &udl.file_stub)?,
                        udl.module_path,
                    ));
                }
                meta => metadata.push(meta.into_node()?),
            }
        }

        let mut root = Root {
            metadata,
            cdylib: Some(path.to_string()),
            ..Root::default()
        };
        for (udl, module_path) in udl_to_load {
            Self::add_metadata_from_udl(&mut root, &udl, &module_path, true)?;
        }
        Ok(root)
    }

    pub fn from_udl(path: &Utf8Path, crate_name: Option<String>) -> Result<Root> {
        let mut root = Root::default();
        let crate_name = match crate_name {
            Some(c) => c,
            None => crate_name_from_cargo_toml(path)?,
        };
        Self::add_metadata_from_udl(&mut root, &fs::read_to_string(path)?, &crate_name, false)?;
        Ok(root)
    }

    fn add_metadata_from_udl(
        root: &mut Root,
        udl: &str,
        crate_name: &str,
        library_mode: bool,
    ) -> Result<()> {
        let metadata_group = uniffi_udl::parse_udl(udl, crate_name)?;
        // parse_udl returns a metadata group, which is nice for the CI, but we actually want to
        // start with a raw metadata list
        if let Some(docstring) = metadata_group.namespace_docstring {
            root.docstrings
                .insert(metadata_group.namespace.name.clone(), docstring);
        }
        root.metadata.extend(
            metadata_group
                .items
                .into_iter()
                // some items are both in UDL and library metadata. For many that's fine but
                // uniffi-traits aren't trivial to compare meaning we end up with dupes.
                // We filter out such problematic items here.
                .filter(|item| {
                    !library_mode || !matches!(item, uniffi_meta::Metadata::UniffiTrait { .. })
                })
                .chain(std::iter::once(uniffi_meta::Metadata::Namespace(
                    metadata_group.namespace,
                )))
                .map(|meta| {
                    Ok(match meta {
                        // Make sure metadata checksums are set
                        uniffi_meta::Metadata::Func(mut func) => {
                            func.checksum = Some(uniffi_meta::checksum(
                                &interface::Function::from(func.clone()),
                            ));
                            uniffi_meta::Metadata::Func(func)
                        }
                        uniffi_meta::Metadata::Method(mut meth) => {
                            meth.checksum = Some(uniffi_meta::checksum(&interface::Method::from(
                                meth.clone(),
                            )));
                            uniffi_meta::Metadata::Method(meth)
                        }
                        uniffi_meta::Metadata::Constructor(mut cons) => {
                            cons.checksum = Some(uniffi_meta::checksum(
                                &interface::Constructor::from(cons.clone()),
                            ));
                            uniffi_meta::Metadata::Constructor(cons)
                        }
                        // Note: UDL-based callbacks don't have checksum functions, don't set the
                        // checksum for those.
                        other => other,
                    })
                })
                .map(|meta| meta.and_then(|meta| meta.into_node()))
                .collect::<Result<Vec<_>>>()?,
        );
        Ok(())
    }
}
