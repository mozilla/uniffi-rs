/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Attribute definitions for a `ComponentInterface`.
//!
//! This module provides some conveniences for working with attribute definitions
//! from WebIDL. When encountering a weedle `ExtendedAttribute` node, use `TryFrom`
//! to convert it into an [`Attribute`] representing one of the attributes that we
//! support. You can also use the [`parse_attributes`] function to parse an
//! `ExtendedAttributeList` into a vec of same.
//!
//! We only support a small number of attributes, so it's manageable to have them
//! all handled by a single abstraction. This might need to be refactored in future
//! if we grow significantly more complicated attribute handling.

use std::convert::TryFrom;

use anyhow::{bail, Result};

/// Represents an attribute parsed from UDL, like [ByRef] or [Throws].
///
/// This is a convenience enum for parsing UDL attributes and erroring out if we encounter
/// any unsupported ones. These don't convert directly into parts of a `ComponentInterface`, but
/// may influence the properties of things like functions and arguments.
#[derive(Debug, Clone, Hash)]
pub(super) enum Attribute {
    ByRef,
    Error,
    Threadsafe,
    Throws(String),
}

impl Attribute {
    pub fn is_error(&self) -> bool {
        matches!(self, Attribute::Error)
    }
}

/// Convert a weedle `ExtendedAttribute` into an `Attribute` for a `ComponentInterface` member,
/// or error out if the attribute is not supported.
impl TryFrom<&weedle::attribute::ExtendedAttribute<'_>> for Attribute {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attribute: &weedle::attribute::ExtendedAttribute<'_>,
    ) -> Result<Self, anyhow::Error> {
        match weedle_attribute {
            // Matches plain named attributes like "[ByRef"].
            weedle::attribute::ExtendedAttribute::NoArgs(attr) => match (attr.0).0 {
                "ByRef" => Ok(Attribute::ByRef),
                "Error" => Ok(Attribute::Error),
                "Threadsafe" => Ok(Attribute::Threadsafe),
                _ => anyhow::bail!("ExtendedAttributeNoArgs not supported: {:?}", (attr.0).0),
            },
            // Matches assignment-style attributes like ["Throws=Error"]
            weedle::attribute::ExtendedAttribute::Ident(identity) => {
                if identity.lhs_identifier.0 == "Throws" {
                    Ok(Attribute::Throws(match identity.rhs {
                        weedle::attribute::IdentifierOrString::Identifier(identifier) => {
                            identifier.0.to_string()
                        }
                        weedle::attribute::IdentifierOrString::String(str_lit) => {
                            str_lit.0.to_string()
                        }
                    }))
                } else {
                    anyhow::bail!(
                        "Attribute identity Identifier not supported: {:?}",
                        identity.lhs_identifier.0
                    )
                }
            }
            _ => anyhow::bail!("Attribute not supported: {:?}", weedle_attribute),
        }
    }
}

/// Parse a weedle `ExtendedAttributeList` into a list of `Attribute`s,
/// erroring out on duplicates.
fn parse_attributes<F>(
    weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    validator: F,
) -> Result<Vec<Attribute>>
where
    F: Fn(&Attribute) -> Result<()>,
{
    let attrs = &weedle_attributes.body.list;

    let mut hash_set = std::collections::HashSet::new();
    for attr in attrs {
        if !hash_set.insert(attr) {
            anyhow::bail!("Duplicated ExtendedAttribute: {:?}", attr);
        }
    }

    let attrs = attrs
        .iter()
        .map(Attribute::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    for attr in attrs.iter() {
        validator(&attr)?;
    }

    Ok(attrs)
}

/// Attributes that can be attached to an `enum` definition in the UDL.
/// There's only one case here: using `[Error]` to mark an enum as an error class.
#[derive(Debug, Clone, Hash, Default)]
pub(super) struct EnumAttributes(Vec<Attribute>);

impl EnumAttributes {
    pub fn contains_error_attr(&self) -> bool {
        self.0.iter().any(|attr| attr.is_error())
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for EnumAttributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    ) -> Result<Self, Self::Error> {
        let attrs = parse_attributes(weedle_attributes, |attr| match attr {
            Attribute::Error => Ok(()),
            _ => bail!(format!("{:?} not supported for enums", attr)),
        })?;
        Ok(Self(attrs))
    }
}

/// Represents UDL attributes that might appear on a function.
///
/// This supports the `[Throws=ErrorName]` attribute for functions that
/// can produce an error.
#[derive(Debug, Clone, Hash, Default)]
pub(super) struct FunctionAttributes(Vec<Attribute>);

impl FunctionAttributes {
    pub(super) fn get_throws_err(&self) -> Option<&str> {
        self.0.iter().find_map(|attr| match attr {
            // This will hopefully return a helpful compilation error
            // if the error is not defined.
            Attribute::Throws(inner) => Some(inner.as_ref()),
            _ => None,
        })
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for FunctionAttributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    ) -> Result<Self, Self::Error> {
        let attrs = parse_attributes(weedle_attributes, |attr| match attr {
            Attribute::Throws(_) => Ok(()),
            _ => bail!(format!(
                "{:?} not supported for functions, methods or constructors",
                attr
            )),
        })?;
        Ok(Self(attrs))
    }
}

/// Represents UDL attributes that might appear on a function argument.
///
/// This supports the `[ByRef]` attribute for arguments that should be passed
/// by reference in the generated Rust scaffolding.
#[derive(Debug, Clone, Hash, Default)]
pub(super) struct ArgumentAttributes(Vec<Attribute>);

impl ArgumentAttributes {
    pub fn by_ref(&self) -> bool {
        self.0.iter().any(|attr| matches!(attr, Attribute::ByRef))
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for ArgumentAttributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    ) -> Result<Self, Self::Error> {
        let attrs = parse_attributes(weedle_attributes, |attr| match attr {
            Attribute::ByRef => Ok(()),
            _ => bail!(format!("{:?} not supported for arguments", attr)),
        })?;
        Ok(Self(attrs))
    }
}

/// Represents UDL attributes that might appear on an `interface` definition.
#[derive(Debug, Clone, Hash, Default)]
pub(super) struct InterfaceAttributes(Vec<Attribute>);

impl InterfaceAttributes {
    pub fn threadsafe(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, Attribute::Threadsafe))
    }
}

impl TryFrom<&weedle::attribute::ExtendedAttributeList<'_>> for InterfaceAttributes {
    type Error = anyhow::Error;
    fn try_from(
        weedle_attributes: &weedle::attribute::ExtendedAttributeList<'_>,
    ) -> Result<Self, Self::Error> {
        let attrs = parse_attributes(weedle_attributes, |attr| match attr {
            Attribute::Threadsafe => Ok(()),
            _ => bail!(format!("{:?} not supported for interface classes", attr)),
        })?;
        Ok(Self(attrs))
    }
}

// There may be some divergence between Methods and Functions at some point,
// but not yet.
pub(super) type MethodAttributes = FunctionAttributes;

// Similarly, currently Constructors only support the same attributes as Functions.
// When this changes, (e.g. https://github.com/mozilla/uniffi-rs/issues/37 to support multiple
// named constructors), ConstructorAttributes will need its own implementation.
pub(super) type ConstructorAttributes = FunctionAttributes;

#[cfg(test)]
mod test {
    use super::*;
    use weedle::Parse;

    #[test]
    fn test_byref() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttribute::parse("ByRef").unwrap();
        let attr = Attribute::try_from(&node)?;
        assert!(matches!(attr, Attribute::ByRef));
        Ok(())
    }

    #[test]
    fn test_error() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttribute::parse("Error").unwrap();
        let attr = Attribute::try_from(&node)?;
        assert!(matches!(attr, Attribute::Error));
        assert!(attr.is_error());
        Ok(())
    }

    #[test]
    fn test_threadsafe() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttribute::parse("Threadsafe").unwrap();
        let attr = Attribute::try_from(&node)?;
        assert!(matches!(attr, Attribute::Threadsafe));
        Ok(())
    }
    #[test]
    fn test_throws() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttribute::parse("Throws=Name").unwrap();
        let attr = Attribute::try_from(&node)?;
        assert!(matches!(attr, Attribute::Throws(nm) if nm == "Name"));

        let (_, node) = weedle::attribute::ExtendedAttribute::parse("Throws").unwrap();
        let err = Attribute::try_from(&node).unwrap_err();
        assert_eq!(
            err.to_string(),
            "ExtendedAttributeNoArgs not supported: \"Throws\""
        );

        Ok(())
    }

    #[test]
    fn test_unsupported() -> Result<()> {
        let (_, node) =
            weedle::attribute::ExtendedAttribute::parse("UnsupportedAttribute").unwrap();
        let err = Attribute::try_from(&node).unwrap_err();
        assert_eq!(
            err.to_string(),
            "ExtendedAttributeNoArgs not supported: \"UnsupportedAttribute\""
        );

        let (_, node) =
            weedle::attribute::ExtendedAttribute::parse("Unsupported=Attribute").unwrap();
        let err = Attribute::try_from(&node).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Attribute identity Identifier not supported: \"Unsupported\""
        );

        Ok(())
    }

    #[test]
    fn test_other_attributes_not_supported_for_enums() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[Error, ByRef]").unwrap();
        let err = EnumAttributes::try_from(&node).unwrap_err();
        assert_eq!(err.to_string(), "ByRef not supported for enums");
        Ok(())
    }

    #[test]
    fn test_throws_attribute() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[Throws=Error]").unwrap();
        let attrs = FunctionAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.get_throws_err(), Some("Error")));

        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[]").unwrap();
        let attrs = FunctionAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.get_throws_err(), None));

        Ok(())
    }

    #[test]
    fn test_other_attributes_not_supported_for_functions() -> Result<()> {
        let (_, node) =
            weedle::attribute::ExtendedAttributeList::parse("[Throws=Error, ByRef]").unwrap();
        let err = FunctionAttributes::try_from(&node).unwrap_err();
        assert_eq!(
            err.to_string(),
            "ByRef not supported for functions, methods or constructors"
        );
        Ok(())
    }

    #[test]
    fn test_byref_attribute() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[ByRef]").unwrap();
        let attrs = ArgumentAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.by_ref(), true));

        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[]").unwrap();
        let attrs = ArgumentAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.by_ref(), false));

        Ok(())
    }

    #[test]
    fn test_other_attributes_not_supported_for_arguments() -> Result<()> {
        let (_, node) =
            weedle::attribute::ExtendedAttributeList::parse("[Throws=Error, ByRef]").unwrap();
        let err = ArgumentAttributes::try_from(&node).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Throws(\"Error\") not supported for arguments"
        );
        Ok(())
    }

    #[test]
    fn test_threadsafe_attribute() -> Result<()> {
        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[Threadsafe]").unwrap();
        let attrs = InterfaceAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.threadsafe(), true));

        let (_, node) = weedle::attribute::ExtendedAttributeList::parse("[]").unwrap();
        let attrs = InterfaceAttributes::try_from(&node).unwrap();
        assert!(matches!(attrs.threadsafe(), false));

        Ok(())
    }

    #[test]
    fn test_other_attributes_not_supported_for_interfaces() -> Result<()> {
        let (_, node) =
            weedle::attribute::ExtendedAttributeList::parse("[Threadsafe, Error]").unwrap();
        let err = InterfaceAttributes::try_from(&node).unwrap_err();
        assert_eq!(err.to_string(), "Error not supported for interface classes");
        Ok(())
    }
}
