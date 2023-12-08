/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains code to handle foreign callbacks - C-ABI functions that are defined by a
//! foreign language, then registered with UniFFI.  These callbacks are used to implement callback
//! interfaces, async scheduling etc. Foreign callbacks are registered at startup, when the foreign
//! code loads the exported library. For each callback type, we also define a "cell" type for
//! storing the callback.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::RustBuffer;

/// ForeignCallback is the Rust representation of a foreign language function.
/// It is the basis for all callbacks interfaces. It is registered exactly once per callback interface,
/// at library start up time.
/// Calling this method is only done by generated objects which mirror callback interfaces objects in the foreign language.
///
/// * The `handle` is the key into a handle map on the other side of the FFI used to look up the foreign language object
///   that implements the callback interface/trait.
/// * The `method` selector specifies the method that will be called on the object, by looking it up in a list of methods from
///   the IDL. The list is 1 indexed. Note that the list of methods is generated by UniFFI from the IDL and used in all
///   bindings, so we can rely on the method list being stable within the same run of UniFFI.
/// * `args_data` and `args_len` represents a serialized buffer of arguments to the function. The scaffolding code
///   writes the callback arguments to this buffer, in order, using `FfiConverter.write()`. The bindings code reads the
///   arguments from the buffer and passes them to the user's callback.
/// * `buf_ptr` is a pointer to where the resulting buffer will be written. UniFFI will allocate a
///   buffer to write the result into.
/// * Callbacks return one of the `CallbackResult` values
///   Note: The output buffer might still contain 0 bytes of data.
pub type ForeignCallback = unsafe extern "C" fn(
    handle: u64,
    method: u32,
    args_data: *const u8,
    args_len: i32,
    buf_ptr: *mut RustBuffer,
) -> i32;

/// Store a [ForeignCallback] pointer
pub(crate) struct ForeignCallbackCell(AtomicUsize);

/// Macro to define foreign callback types as well as the callback cell.
macro_rules! impl_foreign_callback_cell {
    ($callback_type:ident, $cell_type:ident) => {
        // Overly-paranoid sanity checking to ensure that these types are
        // convertible between each-other. `transmute` actually should check this for
        // us too, but this helps document the invariants we rely on in this code.
        //
        // Note that these are guaranteed by
        // https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html
        // and thus this is a little paranoid.
        static_assertions::assert_eq_size!(usize, $callback_type);
        static_assertions::assert_eq_size!(usize, Option<$callback_type>);

        impl $cell_type {
            pub const fn new() -> Self {
                Self(AtomicUsize::new(0))
            }

            pub fn set(&self, callback: $callback_type) {
                // Store the pointer using Ordering::Relaxed.  This is sufficient since callback
                // should be set at startup, before there's any chance of using them.
                self.0.store(callback as usize, Ordering::Relaxed);
            }

            pub fn get(&self) -> $callback_type {
                let ptr_value = self.0.load(Ordering::Relaxed);
                unsafe {
                    // SAFETY: self.0 was set in `set` from our function pointer type, so
                    // it's safe to transmute it back here.
                    ::std::mem::transmute::<usize, Option<$callback_type>>(ptr_value)
                        .expect("Bug: callback not set.  This is likely a uniffi bug.")
                }
            }
        }
    };
}

impl_foreign_callback_cell!(ForeignCallback, ForeignCallbackCell);
