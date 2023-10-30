/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod rustfuture;
mod scheduler;
mod wrappedfuture;

pub use rustfuture::RustFutureFfi;
use rustfuture::*;
use scheduler::*;
use wrappedfuture::*;

use crate::{Handle, LowerReturn, RustCallStatus, SlabAlloc};
use std::{future::Future, sync::Arc};

// === Public FFI API ===

/// Foreign callback that's passed to [rust_future_poll]
///
/// The Rust side of things calls this when the foreign side should call [rust_future_poll] again
/// to continue progress on the future.
pub type RustFutureContinuationCallback = extern "C" fn(callback_data: Handle, RustFuturePoll);

/// Create a new [Handle] for a RustFuture
///
/// For each exported async function, UniFFI will create a scaffolding function that uses this to
/// create the [Handle] to pass to the foreign code.
pub fn rust_future_new<F, T, UT>(future: F, tag: UT) -> Handle
where
    // See the [RustFuture] struct for an explanation of these trait bounds
    F: Future<Output = T> + Send + 'static,
    T: LowerReturn<UT> + Send + 'static,
    UT: Send + 'static,
    // Needed to create a Handle
    dyn RustFutureFfi<T::ReturnType>: SlabAlloc<UT>,
{
    // Create a RustFuture and coerce to `Arc<dyn RustFutureFfi>`, which is what we use to
    // implement the FFI
    <dyn RustFutureFfi<T::ReturnType> as SlabAlloc<UT>>::insert(
        RustFuture::new(future, tag) as Arc<dyn RustFutureFfi<T::ReturnType>>
    )
}

/// Poll a Rust future
///
/// When the future is ready to progress the continuation will be called with the `data` value and
/// a [RustFuturePoll] value. For each [rust_future_poll] call the continuation will be called
/// exactly once.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_poll<ReturnType, UT>(
    handle: Handle,
    callback: RustFutureContinuationCallback,
    data: Handle,
) where
    dyn RustFutureFfi<ReturnType>: SlabAlloc<UT>,
{
    <dyn RustFutureFfi<ReturnType> as SlabAlloc<UT>>::get_clone(handle).ffi_poll(callback, data)
}

/// Cancel a Rust future
///
/// Any current and future continuations will be immediately called with RustFuturePoll::Ready.
///
/// This is needed for languages like Swift, which continuation to wait for the continuation to be
/// called when tasks are cancelled.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_cancel<ReturnType, UT>(handle: Handle)
where
    dyn RustFutureFfi<ReturnType>: SlabAlloc<UT>,
{
    <dyn RustFutureFfi<ReturnType> as SlabAlloc<UT>>::get_clone(handle).ffi_cancel()
}

/// Complete a Rust future
///
/// Note: the actually extern "C" scaffolding functions can't be generic, so we generate one for
/// each supported FFI type.
///
/// # Safety
///
/// - The [Handle] must not previously have been passed to [rust_future_free]
/// - The `T` param must correctly correspond to the [rust_future_new] call.  It must
///   be `<Output as LowerReturn<UT>>::ReturnType`
pub unsafe fn rust_future_complete<ReturnType, UT>(
    handle: Handle,
    out_status: &mut RustCallStatus,
) -> ReturnType
where
    dyn RustFutureFfi<ReturnType>: SlabAlloc<UT>,
{
    <dyn RustFutureFfi<ReturnType> as SlabAlloc<UT>>::get_clone(handle).ffi_complete(out_status)
}

/// Free a Rust future, dropping the strong reference and releasing all references held by the
/// future.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_free<ReturnType, UT>(handle: Handle)
where
    dyn RustFutureFfi<ReturnType>: SlabAlloc<UT>,
{
    <dyn RustFutureFfi<ReturnType> as SlabAlloc<UT>>::remove(handle).ffi_free()
}

#[cfg(test)]
mod tests;
