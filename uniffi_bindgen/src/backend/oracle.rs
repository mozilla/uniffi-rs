/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{CodeType, TypeIdentifier};
use crate::interface::FfiType;

/// An object to supply a foreign language specific CodeType for a given type. It also
/// supplys the specific rendering of a given identifier when used in a specific context.
pub trait CodeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Box<dyn CodeType>;

    /// Get the idiomatic rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &str) -> String;

    /// Get the idiomatic rendering of a function name.
    fn fn_name(&self, nm: &str) -> String;

    /// Get the idiomatic rendering of a variable name.
    fn var_name(&self, nm: &str) -> String;

    /// Get the idiomatic rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &str) -> String;

    /// Get the idiomatic rendering of an error name.
    fn error_name(&self, nm: &str) -> String;

    fn ffi_type_label(&self, ffi_type: &FfiType) -> String;
}
