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
    N,
    S(String),
    I(i8),
}

impl std::fmt::Display for TraitEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::N => write!(f, "TraitEnum::N"),
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
                TraitEnum::I(_) | TraitEnum::N => std::cmp::Ordering::Less,
            },
            TraitEnum::I(_) => match other {
                TraitEnum::S(_) => std::cmp::Ordering::Greater,
                TraitEnum::I(_) => std::cmp::Ordering::Equal,
                TraitEnum::N => std::cmp::Ordering::Less,
            },
            TraitEnum::N => match other {
                TraitEnum::N => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Greater,
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

// flat enum with Display only - Kotlin doesn't support Eq/Ord/Hash exports for flat enums
#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[uniffi::export(Debug, Display)]
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

// flat error - variants have no fields
#[derive(Debug, uniffi::Error, thiserror::Error, Clone, PartialEq, Eq, Hash)]
#[uniffi::export(Display)]
pub enum FlatError {
    #[error("error: not found")]
    NotFound,
    #[error("error: unauthorized")]
    Unauthorized,
    #[error("error: internal error")]
    InternalError,
}

#[uniffi::export]
fn throw_flat_error(i: u8) -> Result<(), FlatError> {
    match i {
        0 => Err(FlatError::NotFound),
        1 => Err(FlatError::Unauthorized),
        _ => Err(FlatError::InternalError),
    }
}

// mixed error with all possible exported traits
#[derive(Debug, uniffi::Error, thiserror::Error, Clone)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub enum MultipleTraitError {
    #[error("MultipleTraitError::NoData")]
    NoData,
    #[error("MultipleTraitError::WithCode({code})")]
    WithCode { code: i32 },
    #[error("MultipleTraitError::AnotherFlat")]
    AnotherFlat,
    #[error("MultipleTraitError::WithMessage({msg})")]
    WithMessage { msg: String },
    #[error("nested simple error: {0}")]
    NestedSimple(FlatError),
    #[error("nested complex error [{tag}]: {inner}")]
    NestedComplex { inner: FlatError, tag: String },
}

// error that doesn't end in "Error" suffix to test class_name filter doesn't break this case
#[derive(Debug, uniffi::Error, thiserror::Error, Clone)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub enum ApiFailure {
    #[error("api network issue")]
    NetworkIssue,
    #[error("api timeout after {duration_ms}ms")]
    Timeout { duration_ms: u32 },
    #[error("api server down")]
    ServerDown,
    #[error("api rate limited: {retry_after}s")]
    RateLimited { retry_after: u32 },
}

impl Ord for ApiFailure {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ApiFailure::NetworkIssue, ApiFailure::NetworkIssue) => std::cmp::Ordering::Equal,
            (ApiFailure::NetworkIssue, _) => std::cmp::Ordering::Less,
            (ApiFailure::Timeout { .. }, ApiFailure::NetworkIssue) => std::cmp::Ordering::Greater,
            (ApiFailure::Timeout { duration_ms: d1 }, ApiFailure::Timeout { duration_ms: d2 }) => {
                d1.cmp(d2)
            }
            (ApiFailure::Timeout { .. }, _) => std::cmp::Ordering::Less,
            (ApiFailure::ServerDown, ApiFailure::RateLimited { .. }) => std::cmp::Ordering::Less,
            (ApiFailure::ServerDown, ApiFailure::ServerDown) => std::cmp::Ordering::Equal,
            (ApiFailure::ServerDown, _) => std::cmp::Ordering::Greater,
            (
                ApiFailure::RateLimited { retry_after: r1 },
                ApiFailure::RateLimited { retry_after: r2 },
            ) => r1.cmp(r2),
            (ApiFailure::RateLimited { .. }, _) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for ApiFailure {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ApiFailure {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for ApiFailure {}

impl Hash for ApiFailure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state)
    }
}

#[uniffi::export]
fn throw_api_failure(i: u8) -> Result<(), ApiFailure> {
    match i {
        0 => Err(ApiFailure::NetworkIssue),
        1 => Err(ApiFailure::Timeout { duration_ms: 5000 }),
        2 => Err(ApiFailure::ServerDown),
        _ => Err(ApiFailure::RateLimited { retry_after: 60 }),
    }
}

#[uniffi::export]
fn throw_api_failure_timeout(duration_ms: u32) -> Result<(), ApiFailure> {
    Err(ApiFailure::Timeout { duration_ms })
}

#[uniffi::export]
fn throw_api_failure_rate_limited(retry_after: u32) -> Result<(), ApiFailure> {
    Err(ApiFailure::RateLimited { retry_after })
}

impl Ord for MultipleTraitError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (MultipleTraitError::NoData, MultipleTraitError::NoData) => std::cmp::Ordering::Equal,
            (MultipleTraitError::NoData, _) => std::cmp::Ordering::Less,
            (MultipleTraitError::WithCode { .. }, MultipleTraitError::NoData) => {
                std::cmp::Ordering::Greater
            }
            (MultipleTraitError::WithCode { .. }, MultipleTraitError::WithCode { .. }) => {
                std::cmp::Ordering::Equal
            }
            (MultipleTraitError::WithCode { .. }, _) => std::cmp::Ordering::Less,
            (MultipleTraitError::AnotherFlat, MultipleTraitError::WithMessage { .. }) => {
                std::cmp::Ordering::Less
            }
            (MultipleTraitError::AnotherFlat, MultipleTraitError::AnotherFlat) => {
                std::cmp::Ordering::Equal
            }
            (MultipleTraitError::AnotherFlat, _) => std::cmp::Ordering::Greater,
            (MultipleTraitError::WithMessage { .. }, MultipleTraitError::WithMessage { .. }) => {
                std::cmp::Ordering::Equal
            }
            (
                MultipleTraitError::WithMessage { .. },
                MultipleTraitError::NestedSimple(_) | MultipleTraitError::NestedComplex { .. },
            ) => std::cmp::Ordering::Less,
            (MultipleTraitError::WithMessage { .. }, _) => std::cmp::Ordering::Greater,
            (MultipleTraitError::NestedSimple(_), MultipleTraitError::NestedSimple(_)) => {
                std::cmp::Ordering::Equal
            }
            (MultipleTraitError::NestedSimple(_), MultipleTraitError::NestedComplex { .. }) => {
                std::cmp::Ordering::Less
            }
            (MultipleTraitError::NestedSimple(_), _) => std::cmp::Ordering::Greater,
            (
                MultipleTraitError::NestedComplex { .. },
                MultipleTraitError::NestedComplex { .. },
            ) => std::cmp::Ordering::Equal,
            (MultipleTraitError::NestedComplex { .. }, _) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for MultipleTraitError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MultipleTraitError {
    fn eq(&self, other: &Self) -> bool {
        Ord::cmp(self, other) == std::cmp::Ordering::Equal
    }
}

impl Eq for MultipleTraitError {}

impl Hash for MultipleTraitError {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state)
    }
}

#[uniffi::export]
fn throw_multiple_trait_error(i: u8) -> Result<(), MultipleTraitError> {
    match i {
        0 => Err(MultipleTraitError::NoData),
        1 => Err(MultipleTraitError::WithCode { code: 42 }),
        2 => Err(MultipleTraitError::AnotherFlat),
        3 => Err(MultipleTraitError::WithMessage {
            msg: "test".to_string(),
        }),
        4 => Err(MultipleTraitError::NestedSimple(FlatError::NotFound)),
        _ => Err(MultipleTraitError::NestedComplex {
            inner: FlatError::Unauthorized,
            tag: "complex".to_string(),
        }),
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
