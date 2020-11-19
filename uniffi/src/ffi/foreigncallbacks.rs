/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::RustBuffer;
use std::sync::atomic::{AtomicUsize, Ordering};

/// ForeignCallback is the function that will do the method dispatch on the foreign language side.
/// It is the basis for all callbacks interfaces. It is registered exactly once per callback interface,
/// at library start up time.
pub type ForeignCallback =
    unsafe extern "C" fn(handle: u64, method: u32, args: RustBuffer) -> RustBuffer;

pub const IDX_CALLBACK_FREE: u32 = 0;

// Overly-paranoid sanity checking to ensure that these types are
// convertible between each-other. `transmute` actually should check this for
// us too, but this helps document the invariants we rely on in this code.
//
// Note that these are guaranteed by
// https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html
// and thus this is a little paranoid.
ffi_support::static_assert!(
    STATIC_ASSERT_USIZE_EQ_FUNC_SIZE,
    std::mem::size_of::<usize>() == std::mem::size_of::<ForeignCallback>()
);

ffi_support::static_assert!(
    STATIC_ASSERT_USIZE_EQ_OPT_FUNC_SIZE,
    std::mem::size_of::<usize>() == std::mem::size_of::<Option<ForeignCallback>>()
);

/// Struct to hold a foreign callback.
pub struct ForeignCallbackInternals {
    callback_ptr: AtomicUsize,
}

impl Default for ForeignCallbackInternals {
    fn default() -> Self {
        ForeignCallbackInternals {
            callback_ptr: AtomicUsize::new(0),
        }
    }
}

impl ForeignCallbackInternals {
    pub const fn new() -> Self {
        ForeignCallbackInternals {
            callback_ptr: AtomicUsize::new(0),
        }
    }

    pub fn set_callback(&self, callback: ForeignCallback) {
        let as_usize = callback as usize;
        let old_ptr = self
            .callback_ptr
            .compare_and_swap(0, as_usize, Ordering::SeqCst);
        if old_ptr != 0 {
            // This is an internal bug, the other side of the FFI should ensure
            // it sets this only once. Note that this is actually going to be
            // before logging is initialized in practice, so there's not a lot
            // we can actually do here.
            panic!("Bug: call set_callback multiple times. This is likely a uniffi bug");
        }
    }

    pub fn get_callback(&self) -> Option<ForeignCallback> {
        let ptr_value = self.callback_ptr.load(Ordering::SeqCst);
        unsafe { std::mem::transmute::<usize, Option<ForeignCallback>>(ptr_value) }
    }
}
