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

// Enums.
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

uniffi::include_scaffolding!("trait_methods");
