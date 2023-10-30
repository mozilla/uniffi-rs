/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{future::Future, sync::Arc};

mod future;
mod scheduler;
use future::*;
use scheduler::*;

#[cfg(test)]
mod tests;

use crate::{LowerReturn, RustCallStatus};

/// Result code for [rust_future_poll].  This is passed to the continuation function.
#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RustFuturePoll {
    /// The future is ready and is waiting for [rust_future_complete] to be called
    Ready = 0,
    /// The future might be ready and [rust_future_poll] should be called again
    MaybeReady = 1,
}

/// Foreign callback that's passed to [rust_future_poll]
///
/// The Rust side of things calls this when the foreign side should call [rust_future_poll] again
/// to continue progress on the future.
pub type RustFutureContinuationCallback = extern "C" fn(callback_data: *const (), RustFuturePoll);

/// Opaque handle for a Rust future that's stored by the foreign language code
#[repr(transparent)]
pub struct RustFutureHandle(*const ());

// === Public FFI API ===

/// Create a new [RustFutureHandle]
///
/// For each exported async function, UniFFI will create a scaffolding function that uses this to
/// create the [RustFutureHandle] to pass to the foreign code.
pub fn rust_future_new<F, T, UT>(future: F, tag: UT) -> RustFutureHandle
where
    // F is the future type returned by the exported async function.  It needs to be Send + `static
    // since it will move between threads for an indeterminate amount of time as the foreign
    // executor calls polls it and the Rust executor wakes it.  It does not need to by `Sync`,
    // since we synchronize all access to the values.
    F: Future<Output = T> + Send + 'static,
    // T is the output of the Future.  It needs to implement [LowerReturn].  Also it must be Send +
    // 'static for the same reason as F.
    T: LowerReturn<UT> + Send + 'static,
    // The UniFfiTag ZST. The Send + 'static bound is to keep rustc happy.
    UT: Send + 'static,
{
    // Create a RustFuture and coerce to `Arc<dyn RustFutureFfi>`, which is what we use to
    // implement the FFI
    let future_ffi = RustFuture::new(future, tag) as Arc<dyn RustFutureFfi<T::ReturnType>>;
    // Box the Arc, to convert the wide pointer into a normal sized pointer so that we can pass it
    // to the foreign code.
    let boxed_ffi = Box::new(future_ffi);
    // We can now create a RustFutureHandle
    RustFutureHandle(Box::into_raw(boxed_ffi) as *mut ())
}

/// Poll a Rust future
///
/// When the future is ready to progress the continuation will be called with the `data` value and
/// a [RustFuturePoll] value. For each [rust_future_poll] call the continuation will be called
/// exactly once.
///
/// # Safety
///
/// The [RustFutureHandle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_poll<ReturnType>(
    handle: RustFutureHandle,
    callback: RustFutureContinuationCallback,
    data: *const (),
) {
    let future = &*(handle.0 as *mut Arc<dyn RustFutureFfi<ReturnType>>);
    future.clone().ffi_poll(callback, data)
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
/// The [RustFutureHandle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_cancel<ReturnType>(handle: RustFutureHandle) {
    let future = &*(handle.0 as *mut Arc<dyn RustFutureFfi<ReturnType>>);
    future.clone().ffi_cancel()
}

/// Complete a Rust future
///
/// Note: the actually extern "C" scaffolding functions can't be generic, so we generate one for
/// each supported FFI type.
///
/// # Safety
///
/// - The [RustFutureHandle] must not previously have been passed to [rust_future_free]
/// - The `T` param must correctly correspond to the [rust_future_new] call.  It must
///   be `<Output as LowerReturn<UT>>::ReturnType`
pub unsafe fn rust_future_complete<ReturnType>(
    handle: RustFutureHandle,
    out_status: &mut RustCallStatus,
) -> ReturnType {
    let future = &*(handle.0 as *mut Arc<dyn RustFutureFfi<ReturnType>>);
    future.ffi_complete(out_status)
}

/// Free a Rust future, dropping the strong reference and releasing all references held by the
/// future.
///
/// # Safety
///
/// The [RustFutureHandle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_free<ReturnType>(handle: RustFutureHandle) {
    let future = Box::from_raw(handle.0 as *mut Arc<dyn RustFutureFfi<ReturnType>>);
    future.ffi_free()
}
