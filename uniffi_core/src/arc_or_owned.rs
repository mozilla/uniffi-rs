/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

/// `Arc<T>` or `T`
///
/// This is used to handle exported functions that return objects.
/// UniFFI allows them to either return `Arc<T>` or `T`.
/// This gives us a trait bound to get the `Arc<T>` in either case
///
/// This is only used by bindgens based on `uniffi_parse_rs`
/// which is currently only the experimental uniffi-bindgen-kotlin-jni.
pub trait ArcOrOwned<T> {
    fn into_arc(self) -> Arc<T>;
}

impl<T> ArcOrOwned<T> for T {
    fn into_arc(self) -> Arc<T> {
        Arc::new(self)
    }
}

impl<T> ArcOrOwned<T> for Arc<T> {
    fn into_arc(self) -> Arc<T> {
        self
    }
}
