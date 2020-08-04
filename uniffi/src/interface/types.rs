/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//#![deny(missing_docs)]
#![allow(unknown_lints)]
#![warn(rust_2018_idioms)]

//! # Our basic typesystem for defining a component interface.

use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::{Attributes, ComponentInterface};

/// Represents all the different types that can be used in a component interface.
/// At this level we identify user-defined types by name, without knowing any details
/// of their internal structure apart from what type of thing they are (record, enum, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeReference {
    Boolean,
    U8,
    S8,
    U16,
    S16,
    U32,
    S32,
    U64,
    S64,
    Float,
    Double,
    String,
    // I don't like this too much since it's not a part of the IDL, and hopefully I can get rid of it
    // But keeping it for the draft until I have a better idea
    // on how to model the freeing. Normal Kotlin functions that take
    // strings take a "String" across the ffi, but the "free" function
    // takes a "Pointer".
    RawStringPointer,
    Bytes,
    Object(String),
    Record(String),
    Enum(String),
    Error(String),
    Optional(Box<TypeReference>),
    Sequence(Box<TypeReference>),
}

fn resolve_builtin_type(id: &weedle::common::Identifier<'_>) -> Option<TypeReference> {
    match id.0 {
        "string" => Some(TypeReference::String),
        "u8" => Some(TypeReference::U8),
        "s8" => Some(TypeReference::S8),
        "u16" => Some(TypeReference::U16),
        "s16" => Some(TypeReference::S16),
        "u32" => Some(TypeReference::U32),
        "s32" => Some(TypeReference::S32),
        "u64" => Some(TypeReference::U64),
        "s64" => Some(TypeReference::S64),
        _ => None,
    }
}

pub(crate) trait TypeFinder {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()>;
}

// Hrm, maybe this should be just a normal method on the ComponentInterface
// rather than  fancy trait? We'll see how it goes...
pub(crate) trait TypeResolver {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference>;
}

impl<T: TypeFinder> TypeFinder for Vec<T> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        for item in self.iter() {
            (&item).find_type_definitions(ci)?;
        }
        Ok(())
    }
}

impl TypeFinder for weedle::Definition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        match self {
            weedle::Definition::Interface(d) => d.find_type_definitions(ci),
            weedle::Definition::Dictionary(d) => d.find_type_definitions(ci),
            weedle::Definition::Enum(d) => d.find_type_definitions(ci),
            weedle::Definition::Typedef(d) => d.find_type_definitions(ci),
            _ => Ok(()),
        }
    }
}

impl TypeFinder for weedle::InterfaceDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        let name = self.identifier.0.to_string();
        ci.add_type_definition(self.identifier.0, TypeReference::Object(name))
    }
}

impl TypeFinder for weedle::DictionaryDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        let name = self.identifier.0.to_string();
        ci.add_type_definition(self.identifier.0, TypeReference::Record(name))
    }
}

impl TypeFinder for weedle::EnumDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        if let Some(attrs) = &self.attributes {
            let attrs = Attributes::try_from(attrs)?;
            if attrs.contains_error_attr() {
                let name = self.identifier.0.to_string();
                return ci.add_type_definition(self.identifier.0, TypeReference::Error(name));
            }
        }
        let name = self.identifier.0.to_string();
        ci.add_type_definition(self.identifier.0, TypeReference::Enum(name))
    }
}

impl TypeFinder for weedle::TypedefDefinition<'_> {
    fn find_type_definitions(&self, ci: &mut ComponentInterface) -> Result<()> {
        if self.attributes.is_some() {
            bail!("no typedef attributes are currently supported");
        }
        match resolve_builtin_type(&self.identifier) {
            Some(_) => bail!(
                "please don't try to redefine builtin types, kthxbye ({:?})",
                self
            ),
            None => {
                // For now, we assume that the typedef must refer to an already-defined type, which means
                // we can look it up in the ComponentInterface. This should suffice for our needs for
                // a good long while before we consider implementing a more complex resolution strategy.
                ci.add_type_definition(self.identifier.0, self.type_.resolve_type_definition(ci)?)
            }
        }
    }
}

impl TypeResolver for weedle::types::Type<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        match self {
            weedle::types::Type::Single(t) => match t {
                weedle::types::SingleType::Any(_) => bail!("no support for `any` types"),
                weedle::types::SingleType::NonAny(t) => t.resolve_type_definition(ci),
            },
            weedle::types::Type::Union(_) => bail!("no support for union types yet"),
        }
    }
}

impl TypeResolver for weedle::types::NonAnyType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        match self {
            weedle::types::NonAnyType::Boolean(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Identifier(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Integer(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::FloatingPoint(t) => t.resolve_type_definition(ci),
            weedle::types::NonAnyType::Sequence(t) => t.resolve_type_definition(ci),
            _ => bail!("no support for type {:?}", self),
        }
    }
}

impl TypeResolver for weedle::types::AttributedNonAnyType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_definition(ci)
    }
}

impl TypeResolver for weedle::types::AttributedType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_definition(ci)
    }
}

impl<T: TypeResolver> TypeResolver for weedle::types::MayBeNull<T> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        let type_ = self.type_.resolve_type_definition(ci)?;
        Ok(match self.q_mark {
            None => type_,
            Some(_) => TypeReference::Optional(Box::new(type_)),
        })
    }
}

impl TypeResolver for weedle::types::IntegerType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<TypeReference> {
        bail!(
            "integer types not implemented ({:?}); consider using u8, u16, u32 or u64",
            self
        )
    }
}

impl TypeResolver for weedle::types::FloatingPointType {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        match self {
            weedle::types::FloatingPointType::Float(t) => t.resolve_type_definition(ci),
            weedle::types::FloatingPointType::Double(t) => t.resolve_type_definition(ci),
        }
    }
}

impl TypeResolver for weedle::types::SequenceType<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        Ok(TypeReference::Sequence(Box::new(
            self.generics.body.as_ref().resolve_type_definition(ci)?,
        )))
    }
}

impl TypeResolver for weedle::common::Identifier<'_> {
    fn resolve_type_definition(&self, ci: &ComponentInterface) -> Result<TypeReference> {
        match resolve_builtin_type(self) {
            Some(type_) => Ok(type_),
            None => match ci.get_type_definition(self.0) {
                Some(type_) => Ok(type_),
                None => bail!("unknown type reference: {}", self.0),
            },
        }
    }
}

impl TypeResolver for weedle::term::Boolean {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<TypeReference> {
        Ok(TypeReference::Boolean)
    }
}

impl TypeResolver for weedle::types::FloatType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<TypeReference> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted float`");
        }
        Ok(TypeReference::Float)
    }
}

impl TypeResolver for weedle::types::DoubleType {
    fn resolve_type_definition(&self, _ci: &ComponentInterface) -> Result<TypeReference> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted double`");
        }
        Ok(TypeReference::Double)
    }
}
