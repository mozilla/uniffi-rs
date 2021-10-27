/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

/// String struct for ComponentInterface items
///
/// This struct has a few advantages over the `String`:
///   - Fast to clone, which we do a lot of
///   - Has convenience methods for commonly used operations
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CIString(Rc<String>);

impl CIString {
    //! Join multiple parts into a CIString with underscores
    pub fn from_parts(parts: Vec<&str>) -> Self {
        parts.join("_").into()
    }
}

impl<'a> From<&'a str> for CIString {
    fn from(val: &'a str) -> CIString {
        CIString(val.to_string().into())
    }
}

impl From<String> for CIString {
    fn from(val: String) -> CIString {
        CIString(val.into())
    }
}

impl From<CIString> for String {
    fn from(val: CIString) -> String {
        val.0.to_string()
    }
}

impl std::fmt::Display for CIString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl std::cmp::PartialEq<str> for CIString {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl std::cmp::PartialEq<CIString> for str {
    fn eq(&self, other: &CIString) -> bool {
        other.0.as_ref() == self
    }
}

impl std::cmp::PartialEq<&str> for CIString {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == other
    }
}

impl std::cmp::PartialEq<CIString> for &str {
    fn eq(&self, other: &CIString) -> bool {
        other.0.as_ref() == self
    }
}

impl std::ops::Deref for CIString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
