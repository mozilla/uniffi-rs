/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::RustBuffer;
use std::sync::atomic::{AtomicUsize, Ordering};

// Mechanics behind callback interfaces.

/// ForeignCallback is the function that will do the method dispatch on the foreign language side.
/// It is the basis for all callbacks interfaces. It is registered exactly once per callback interface,
/// at library start up time.
pub type ForeignCallback =
    unsafe extern "C" fn(handle: u64, method: u32, args: RustBuffer) -> RustBuffer;

/// Set the function pointer to the ForeignCallback. Returns false if we did nothing because the callback had already been initialized
pub fn set_foreign_callback(callback_ptr: &AtomicUsize, h: ForeignCallback) -> bool {
    let as_usize = h as usize;
    let old_ptr = callback_ptr.compare_and_swap(0, as_usize, Ordering::SeqCst);
    if old_ptr != 0 {
        // This is an internal bug, the other side of the FFI should ensure
        // it sets this only once. Note that this is actually going to be
        // before logging is initialized in practice, so there's not a lot
        // we can actually do here.
        log::error!("Bug: Initialized CALLBACK_PTR multiple times");
    }
    old_ptr == 0
}

/// Get the function pointer to the ForeignCallback. Panics if the callback
/// has not yet been initialized.
pub fn get_foreign_callback(callback_ptr: &AtomicUsize) -> Option<ForeignCallback> {
    let ptr_value = callback_ptr.load(Ordering::SeqCst);
    unsafe { std::mem::transmute::<usize, Option<ForeignCallback>>(ptr_value) }
}
