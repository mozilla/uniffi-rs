/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! uniffi-bindgen-kotlin-jni-runtime
//!
//! Shared/generic code used by uniffi-bindgen-kotlin-jni consumers

mod caching;
mod calls;
mod strings;

pub use caching::*;
pub use calls::*;
pub use jni_sys::*;
pub use strings::*;
// Re-export for consumers
pub use uniffi;
