/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::Result;

/// Trait for custom type conversions
///
/// This tracks the `try_from` and `into` conversions.
/// Like the other FFI traits, it's paramaterized by a `UniffiTag` type.
/// This allows crates to implement the trait for remote types.
///
/// This is only used by bindgens based on `uniffi_parse_rs`
/// which is currently only the experimental uniffi-bindgen-kotlin-jni.
pub trait CustomType<T> {
    type Builtin;

    fn lower(value: Self) -> Self::Builtin;
    fn try_lift(value: Self::Builtin) -> Result<Self>
    where
        Self: Sized;
}
