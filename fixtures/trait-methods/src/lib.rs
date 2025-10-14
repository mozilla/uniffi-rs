/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitMethods {
    val: String,
}

impl TraitMethods {
    fn new(val: String) -> Self {
        Self { val }
    }
}

impl PartialOrd for TraitMethods {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TraitMethods {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl std::fmt::Display for TraitMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TraitMethods({})", self.val)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, uniffi::Object)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub struct ProcTraitMethods {
    val: String,
}

#[uniffi::export]
impl ProcTraitMethods {
    #[uniffi::constructor]
    fn new(val: String) -> Arc<Self> {
        Arc::new(Self { val })
    }
}

impl PartialOrd for ProcTraitMethods {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcTraitMethods {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl std::fmt::Display for ProcTraitMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProcTraitMethods({})", self.val)
    }
}

// Enums and Records.
// Our bindings will often auto-generate the local equivalent of `Eq` etc, so if we
// just `#[derive(Eq)` etc here , our tests would see the same results even if our versions weren't hoooked up.
// So we need to implement a non-obvious implementation to test against.
// Records
#[derive(uniffi::Record, Debug)]
#[uniffi::export(Debug, Eq, Ord, Hash)]
pub struct TraitRecord {
    s: String,
    i: i8,
}

// only compare the string, not the int
impl Ord for TraitRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.s, &other.s)
    }
}

impl PartialOrd for TraitRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TraitRecord {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for TraitRecord {}

impl Hash for TraitRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.s.hash(state)
    }
}

#[derive(Debug)]
pub struct UdlRecord {
    s: String,
    i: i8,
}

// only compare the string, not the int
impl Ord for UdlRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.s, &other.s)
    }
}

impl PartialOrd for UdlRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UdlRecord {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for UdlRecord {}

impl Hash for UdlRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.s.hash(state)
    }
}

// Enums
#[derive(uniffi::Enum, Debug)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub enum TraitEnum {
    S(String),
    I(i8),
}

impl std::fmt::Display for TraitEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S(s) => write!(f, "TraitEnum::S({s:?})"),
            Self::I(i) => write!(f, "TraitEnum::I({i})"),
        }
    }
}

// only compare the variant, not the content
impl Ord for TraitEnum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // no `Ord` for `std::mem::discriminant`?
        match self {
            TraitEnum::S { .. } => match other {
                TraitEnum::S(_) => std::cmp::Ordering::Equal,
                TraitEnum::I(_) => std::cmp::Ordering::Less,
            },
            TraitEnum::I(_) => match other {
                TraitEnum::S(_) => std::cmp::Ordering::Greater,
                TraitEnum::I(_) => std::cmp::Ordering::Equal,
            },
        }
    }
}
impl PartialOrd for TraitEnum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TraitEnum {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for TraitEnum {}

impl Hash for TraitEnum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state)
    }
}

#[derive(Debug)]
pub enum UdlEnum {
    S { s: String },
    I { i: i8 },
}

impl std::fmt::Display for UdlEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S { s } => write!(f, "UdlEnum::S {{ s: {s:?} }}"),
            Self::I { i } => write!(f, "UdlEnum::I {{ i: {i} }}"),
        }
    }
}

// only compare the variant, not the content
impl Ord for UdlEnum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // no `Ord` for `std::mem::discriminant`?
        match self {
            UdlEnum::S { .. } => match other {
                UdlEnum::S { .. } => std::cmp::Ordering::Equal,
                UdlEnum::I { .. } => std::cmp::Ordering::Less,
            },
            UdlEnum::I { .. } => match other {
                UdlEnum::S { .. } => std::cmp::Ordering::Greater,
                UdlEnum::I { .. } => std::cmp::Ordering::Equal,
            },
        }
    }
}
impl PartialOrd for UdlEnum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UdlEnum {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for UdlEnum {}

impl Hash for UdlEnum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state)
    }
}

// flat enum with Display
#[derive(uniffi::Enum, Debug, Clone)]
#[uniffi::export(Display)]
pub enum EnumWithDisplayExport {
    One,
    Two,
    Three,
}

impl std::fmt::Display for EnumWithDisplayExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "display: One"),
            Self::Two => write!(f, "display: Two"),
            Self::Three => write!(f, "display: Three"),
        }
    }
}

#[uniffi::export]
fn get_enum_with_display_export(i: u8) -> EnumWithDisplayExport {
    match i {
        0 => EnumWithDisplayExport::One,
        1 => EnumWithDisplayExport::Two,
        _ => EnumWithDisplayExport::Three,
    }
}

// nested enum with another enum that implements Display as a payload
#[derive(uniffi::Enum, Debug, Clone)]
#[uniffi::export(Display)]
pub enum NestedEnumWithDisplay {
    Simple(EnumWithDisplayExport),
    Complex {
        inner: EnumWithDisplayExport,
        tag: String,
    },
}

impl std::fmt::Display for NestedEnumWithDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple(e) => write!(f, "nested simple: {}", e),
            Self::Complex { inner, tag } => write!(f, "nested complex [{}]: {}", tag, inner),
        }
    }
}

#[uniffi::export]
fn get_nested_enum_with_display(i: u8) -> NestedEnumWithDisplay {
    match i {
        0 => NestedEnumWithDisplay::Simple(EnumWithDisplayExport::One),
        1 => NestedEnumWithDisplay::Complex {
            inner: EnumWithDisplayExport::Two,
            tag: "test".to_string(),
        },
        _ => NestedEnumWithDisplay::Simple(EnumWithDisplayExport::Three),
    }
}

// flat enum exporting Eq/Ord/Hash - Kotlin enum class provides these natively
#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub enum FlatTraitEnum {
    Alpha,
    Beta,
    Gamma,
}

impl std::fmt::Display for FlatTraitEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            FlatTraitEnum::Alpha => "flat-alpha",
            FlatTraitEnum::Beta => "flat-beta",
            FlatTraitEnum::Gamma => "flat-gamma",
        };
        write!(f, "FlatTraitEnum::{label}")
    }
}

#[uniffi::export]
fn get_flat_trait_enum(index: u8) -> FlatTraitEnum {
    match index {
        0 => FlatTraitEnum::Alpha,
        1 => FlatTraitEnum::Beta,
        _ => FlatTraitEnum::Gamma,
    }
}

// flat enum with explicit numeric discriminant and Display
#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
#[uniffi::export(Display)]
#[repr(u16)]
pub enum NumericEnum {
    Red = 100,
    Green = 200,
    Blue = 300,
}

impl std::fmt::Display for NumericEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            NumericEnum::Red => "color-red",
            NumericEnum::Green => "color-green",
            NumericEnum::Blue => "color-blue",
        };
        write!(f, "NumericEnum::{label}")
    }
}

#[uniffi::export]
fn get_numeric_enum(value: u16) -> NumericEnum {
    match value {
        100 => NumericEnum::Red,
        200 => NumericEnum::Green,
        _ => NumericEnum::Blue,
    }
}

// flat error with Display
#[derive(Debug, uniffi::Error, thiserror::Error, Clone)]
#[uniffi::export(Display)]
pub enum FlatErrorWithDisplayExport {
    #[error("display: too many items: {count}")]
    TooMany { count: u32 },
    #[error("display: too few items: {count}")]
    TooFew { count: u32 },
}

#[uniffi::export]
fn throw_trait_error(i: u8) -> Result<(), FlatErrorWithDisplayExport> {
    match i {
        0 => Err(FlatErrorWithDisplayExport::TooMany { count: 100 }),
        _ => Err(FlatErrorWithDisplayExport::TooFew { count: 0 }),
    }
}

// nested error with another error that implements Display as a payload
#[derive(Debug, uniffi::Error, thiserror::Error, Clone)]
#[uniffi::export(Display)]
pub enum NestedErrorWithDisplay {
    #[error("nested simple error: {0}")]
    Simple(FlatErrorWithDisplayExport),
    #[error("nested complex error [{tag}]: {inner}")]
    Complex {
        inner: FlatErrorWithDisplayExport,
        tag: String,
    },
}

#[uniffi::export]
fn throw_nested_error(i: u8) -> Result<(), NestedErrorWithDisplay> {
    match i {
        0 => Err(NestedErrorWithDisplay::Simple(
            FlatErrorWithDisplayExport::TooMany { count: 42 },
        )),
        1 => Err(NestedErrorWithDisplay::Complex {
            inner: FlatErrorWithDisplayExport::TooFew { count: 7 },
            tag: "nested".to_string(),
        }),
        _ => Err(NestedErrorWithDisplay::Simple(
            FlatErrorWithDisplayExport::TooFew { count: 0 },
        )),
    }
}

// mixed enum with object variants (no fields) and data class variants (with fields)
// this tests that parent-level trait implementations work for object variants
#[derive(uniffi::Enum, Debug, Clone)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub enum MixedEnum {
    NoFields,                  // object variant - needs parent-level trait impls
    WithString(String),        // data class variant - has its own trait impls
    AnotherNoFields,           // object variant - needs parent-level trait impls
    WithNumber { value: i32 }, // data class variant - has its own trait impls
}

impl std::fmt::Display for MixedEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFields => write!(f, "MixedEnum::NoFields"),
            Self::WithString(s) => write!(f, "MixedEnum::WithString({s})"),
            Self::AnotherNoFields => write!(f, "MixedEnum::AnotherNoFields"),
            Self::WithNumber { value } => write!(f, "MixedEnum::WithNumber({value})"),
        }
    }
}

// only compare by discriminant, ignore payloads
impl Ord for MixedEnum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (MixedEnum::NoFields, MixedEnum::NoFields) => std::cmp::Ordering::Equal,
            (MixedEnum::NoFields, _) => std::cmp::Ordering::Less,
            (MixedEnum::WithString(_), MixedEnum::NoFields) => std::cmp::Ordering::Greater,
            (MixedEnum::WithString(_), MixedEnum::WithString(_)) => std::cmp::Ordering::Equal,
            (MixedEnum::WithString(_), _) => std::cmp::Ordering::Less,
            (MixedEnum::AnotherNoFields, MixedEnum::WithNumber { .. }) => std::cmp::Ordering::Less,
            (MixedEnum::AnotherNoFields, MixedEnum::AnotherNoFields) => std::cmp::Ordering::Equal,
            (MixedEnum::AnotherNoFields, _) => std::cmp::Ordering::Greater,
            (MixedEnum::WithNumber { .. }, MixedEnum::WithNumber { .. }) => {
                std::cmp::Ordering::Equal
            }
            (MixedEnum::WithNumber { .. }, _) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for MixedEnum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MixedEnum {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for MixedEnum {}

impl Hash for MixedEnum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state)
    }
}

#[uniffi::export]
fn get_mixed_enum(i: u8) -> MixedEnum {
    match i {
        0 => MixedEnum::NoFields,
        1 => MixedEnum::WithString("test".to_string()),
        2 => MixedEnum::AnotherNoFields,
        _ => MixedEnum::WithNumber { value: 42 },
    }
}

#[cfg(test)]
// make sure the semantics are what we expect locally.
#[test]
fn test_traitenum_traits() {
    let s1 = TraitEnum::S("s1".to_string());
    assert_eq!(format!("{s1:?}"), "S(\"s1\")");
    assert_eq!(format!("{s1}"), "TraitEnum::S(\"s1\")");

    // ord/eq etc
    assert_eq!(Ord::cmp(&s1, &s1), std::cmp::Ordering::Equal);
    assert_eq!(s1, s1);
    // compare equal with different data.
    assert_eq!(
        Ord::cmp(&s1, &TraitEnum::S("s2".to_string())),
        std::cmp::Ordering::Equal
    );
    assert_eq!(
        Ord::cmp(&TraitEnum::I(0), &TraitEnum::I(1)),
        std::cmp::Ordering::Equal
    );
    assert_eq!(TraitEnum::I(0), TraitEnum::I(1));
    assert_ne!(s1, TraitEnum::I(0));
    assert!(s1 < TraitEnum::I(0));
}

uniffi::include_scaffolding!("trait_methods");
