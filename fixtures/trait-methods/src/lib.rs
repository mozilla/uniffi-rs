/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

uniffi::include_scaffolding!("trait_methods");
