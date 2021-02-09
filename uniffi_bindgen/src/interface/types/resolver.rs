/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! #  Helpers for resolving UDL type expressions into concrete types.
//!
//! This module provides the [`TypeResolver`] trait, an abstraction for walking
//! the parse tree of a weedle type expression and using a [`TypeUniverse`] to
//! convert it into a concrete type definition (so it assumes that you're already
//! used a [`TypeFinder`] to populate the universe).
//!
//! Perhaps most importantly, it knows how to error out if the UDL tries to reference
//! an undefined or invalid type.

use anyhow::{bail, Result};

use super::{Type, TypeUniverse};

/// Trait to help resolving an UDL type node to a [`Type`].
///
/// Ths trait does structural matching against type-related weedle AST nodes from
/// a parsed UDL file, turning them into a corresponding [`Type`] struct. It uses the
/// known type definitions in a [`TypeUniverse`] to resolve names to types.
///
/// As a side-effect, resolving a type expression will grow the type universe with
/// references to the types seem during traversal. For example resolving the type
/// expression "sequence<TestRecord>?" will:
///
///   * add `Optional<Sequence<TestRecord>` and `Sequence<TestRecord>` to the
///     known types in the universe.
///   * error out if the type name `TestRecord` is not already known.
///
pub(crate) trait TypeResolver {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type>;
}

impl TypeResolver for &weedle::types::Type<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        (*self).resolve_type_expression(types)
    }
}

impl TypeResolver for weedle::types::Type<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::Type::Single(t) => match t {
                weedle::types::SingleType::Any(_) => bail!("no support for `any` types"),
                weedle::types::SingleType::NonAny(t) => t.resolve_type_expression(types),
            },
            weedle::types::Type::Union(_) => bail!("no support for union types yet"),
        }
    }
}

impl TypeResolver for weedle::types::NonAnyType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::NonAnyType::Boolean(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Identifier(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Integer(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::FloatingPoint(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::Sequence(t) => t.resolve_type_expression(types),
            weedle::types::NonAnyType::RecordType(t) => t.resolve_type_expression(types),
            _ => bail!("no support for type {:?}", self),
        }
    }
}

impl TypeResolver for &weedle::types::AttributedNonAnyType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_expression(types)
    }
}

impl TypeResolver for &weedle::types::AttributedType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.attributes.is_some() {
            bail!("type attributes are not supported yet");
        }
        (&self.type_).resolve_type_expression(types)
    }
}

impl<T: TypeResolver> TypeResolver for weedle::types::MayBeNull<T> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let type_ = self.type_.resolve_type_expression(types)?;
        match self.q_mark {
            None => Ok(type_),
            Some(_) => types.add_known_type(Type::Optional(Box::new(type_))),
        }
    }
}

impl TypeResolver for weedle::types::IntegerType {
    fn resolve_type_expression(&self, _types: &mut TypeUniverse) -> Result<Type> {
        bail!(
            "WebIDL integer types not implemented ({:?}); consider using u8, u16, u32 or u64",
            self
        )
    }
}

impl TypeResolver for weedle::types::FloatingPointType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match self {
            weedle::types::FloatingPointType::Float(t) => t.resolve_type_expression(types),
            weedle::types::FloatingPointType::Double(t) => t.resolve_type_expression(types),
        }
    }
}

impl TypeResolver for weedle::types::SequenceType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let t = self.generics.body.as_ref().resolve_type_expression(types)?;
        types.add_known_type(Type::Sequence(Box::new(t)))
    }
}

impl TypeResolver for weedle::types::RecordType<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        let t = (&self.generics.body.2).resolve_type_expression(types)?;
        // Maps always have string keys, make sure the `String` type is known.
        types.add_known_type(Type::String)?;
        types.add_known_type(Type::Map(Box::new(t)))
    }
}

impl TypeResolver for weedle::common::Identifier<'_> {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        match resolve_builtin_type(self.0) {
            Some(type_) => types.add_known_type(type_),
            None => match types.get_type_definition(self.0) {
                Some(type_) => types.add_known_type(type_),
                None => bail!("unknown type reference: {}", self.0),
            },
        }
    }
}

impl TypeResolver for weedle::term::Boolean {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        types.add_known_type(Type::Boolean)
    }
}

impl TypeResolver for weedle::types::FloatType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted float`");
        }
        types.add_known_type(Type::Float32)
    }
}

impl TypeResolver for weedle::types::DoubleType {
    fn resolve_type_expression(&self, types: &mut TypeUniverse) -> Result<Type> {
        if self.unrestricted.is_some() {
            bail!("we don't support `unrestricted double`");
        }
        types.add_known_type(Type::Float64)
    }
}

/// Resolve built-in API types by name.
///
/// Given an identifier from the UDL, this will return `Some(Type)` if it names one of the
/// built-in primitive types or `None` if it names something else.
pub(in super::super) fn resolve_builtin_type(name: &str) -> Option<Type> {
    match name {
        "string" => Some(Type::String),
        "u8" => Some(Type::UInt8),
        "i8" => Some(Type::Int8),
        "u16" => Some(Type::UInt16),
        "i16" => Some(Type::Int16),
        "u32" => Some(Type::UInt32),
        "i32" => Some(Type::Int32),
        "u64" => Some(Type::UInt64),
        "i64" => Some(Type::Int64),
        "f32" => Some(Type::Float32),
        "f64" => Some(Type::Float64),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use weedle::Parse;

    #[test]
    fn test_named_type_resolution() -> Result<()> {
        let mut types = TypeUniverse::default();
        types.add_type_definition("TestRecord", Type::Record("TestRecord".into()))?;
        assert_eq!(types.iter_known_types().count(), 1);

        let (_, expr) = weedle::types::Type::parse("TestRecord").unwrap();
        let t = types.resolve_type_expression(expr).unwrap();
        assert!(matches!(t, Type::Record(nm) if nm == "TestRecord"));
        assert_eq!(types.iter_known_types().count(), 1);

        let (_, expr) = weedle::types::Type::parse("TestRecord?").unwrap();
        let t = types.resolve_type_expression(expr).unwrap();
        assert!(matches!(t, Type::Optional(_)));
        // Matching the Box<T> is hard, use names as a convenient workaround.
        assert_eq!(t.canonical_name(), "OptionalRecordTestRecord");
        assert_eq!(types.iter_known_types().count(), 2);

        Ok(())
    }

    #[test]
    fn test_resolving_optional_type_adds_inner_type() -> Result<()> {
        let mut types = TypeUniverse::default();
        assert_eq!(types.iter_known_types().count(), 0);
        let (_, expr) = weedle::types::Type::parse("u32?").unwrap();
        let t = types.resolve_type_expression(expr).unwrap();
        assert_eq!(t.canonical_name(), "Optionalu32");
        assert_eq!(types.iter_known_types().count(), 2);
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "u32")
            .is_some());
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "Optionalu32")
            .is_some());
        Ok(())
    }

    #[test]
    fn test_resolving_sequence_type_adds_inner_type() -> Result<()> {
        let mut types = TypeUniverse::default();
        assert_eq!(types.iter_known_types().count(), 0);
        let (_, expr) = weedle::types::Type::parse("sequence<string>").unwrap();
        let t = types.resolve_type_expression(expr).unwrap();
        assert_eq!(t.canonical_name(), "Sequencestring");
        assert_eq!(types.iter_known_types().count(), 2);
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "Sequencestring")
            .is_some());
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "string")
            .is_some());
        Ok(())
    }

    #[test]
    fn test_resolving_map_type_adds_string_and_inner_type() -> Result<()> {
        let mut types = TypeUniverse::default();
        assert_eq!(types.iter_known_types().count(), 0);
        let (_, expr) = weedle::types::Type::parse("record<DOMString, float>").unwrap();
        let t = types.resolve_type_expression(expr).unwrap();
        assert_eq!(t.canonical_name(), "Mapf32");
        assert_eq!(types.iter_known_types().count(), 3);
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "Mapf32")
            .is_some());
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "string")
            .is_some());
        assert!(types
            .iter_known_types()
            .find(|t| t.canonical_name() == "f32")
            .is_some());
        Ok(())
    }

    #[test]
    fn test_error_on_unknown_type() -> Result<()> {
        let mut types = TypeUniverse::default();
        types.add_type_definition("TestRecord", Type::Record("TestRecord".into()))?;
        // Oh no, someone made a typo in the type-o...
        let (_, expr) = weedle::types::Type::parse("TestRecrd").unwrap();
        let err = types.resolve_type_expression(expr).unwrap_err();
        assert_eq!(err.to_string(), "unknown type reference: TestRecrd");
        Ok(())
    }

    #[test]
    fn test_error_on_union_type() -> Result<()> {
        let mut types = TypeUniverse::default();
        types.add_type_definition("TestRecord", Type::Record("TestRecord".into()))?;
        let (_, expr) = weedle::types::Type::parse("(TestRecord or u32)").unwrap();
        let err = types.resolve_type_expression(expr).unwrap_err();
        assert_eq!(err.to_string(), "no support for union types yet");
        Ok(())
    }
}
