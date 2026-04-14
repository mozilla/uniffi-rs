/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Eq, uniffi::Record)]
#[uniffi::export(Debug, Display, Eq, Ord, Hash)]
pub struct RustTraitTest {
    pub a: i32,
    pub b: i32,
}

impl fmt::Debug for RustTraitTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "debug-test-string")
    }
}

impl fmt::Display for RustTraitTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "display-test-string")
    }
}

// Only include `a` in the hash/ord/eq impls
// This gives us a way to test this in the foreign bindings.

impl Hash for RustTraitTest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.a.hash(state);
    }
}

impl PartialEq<RustTraitTest> for RustTraitTest {
    fn eq(&self, other: &RustTraitTest) -> bool {
        self.a == other.a
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd<RustTraitTest> for RustTraitTest {
    fn partial_cmp(&self, other: &RustTraitTest) -> Option<Ordering> {
        Some(self.a.cmp(&other.a))
    }
}

impl Ord for RustTraitTest {
    fn cmp(&self, other: &RustTraitTest) -> Ordering {
        self.a.cmp(&other.a)
    }
}

#[uniffi::export]
pub fn rust_trait_test_hash(trait_test: RustTraitTest) -> u64 {
    let mut s = DefaultHasher::new();
    Hash::hash(&trait_test, &mut s);
    s.finish()
}

#[derive(uniffi::Record)]
#[uniffi::export(Debug)]
pub struct RustTraitTest2 {
    pub a: i32,
    pub b: i32,
}

impl fmt::Debug for RustTraitTest2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "debug-test-string")
    }
}
