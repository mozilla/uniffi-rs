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

use crate::{derive_ffi_traits, Handle, HandleAlloc, LiftArgsError, LowerReturn, RustCallStatus};

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
pub type RustFutureContinuationCallback = extern "C" fn(callback_data: u64, RustFuturePoll);

/// This marker trait allows us to put different bounds on the `Future`s we
/// support, based on `#[cfg(..)]` configuration.
///
/// It should not be considered as a part of the public API, and as such as
/// an implementation detail and subject to change.
///
/// It is _not_ intended to be implemented by libray users or bindings
/// implementors.
#[doc(hidden)]
pub trait UniffiCompatibleFuture<T>: Future<Output = T> {}

/// The `Send` bound is required because the Foreign code may call the
/// `rust_future_*` methods from different threads.
#[cfg(not(target_arch = "wasm32"))]
impl<T, F> UniffiCompatibleFuture<T> for F where F: Future<Output = T> + Send {}

/// `Future`'s on WASM32 are not `Send` because it's a single threaded environment.
///
/// # Safety:
///
/// WASM32 is a single threaded environment. However, in a browser there do
/// exist [`WebWorker`][webworker]s which do not share memory or event-loop
/// with the main browser context.
///
/// Communication between contexts is only possible by message passing,
/// using a small number of ['transferable' object types][transferable].
///
/// The most common source of asynchrony in Rust compiled to WASM is
/// [wasm-bindgen's `JsFuture`][jsfuture]. It is not `Send` because:
///
/// 1. `T` and `E` are both `JsValue`
/// 2. `JsValue` may contain `JsFunction`s, either as a function themselves or
///    an object containing functions.
/// 3. Functions cannot be [serialized and sent][transferable] to `WebWorker`s.
///
/// Implementors of binding generators should be able to enumerate the
/// combinations of Rust or JS communicating across different contexts (here
/// using: <->), and in the same context (+) to account for why it is safe
/// for UniFFI to support `Future`s that are not `Send`:
///
/// 1. JS + Rust in the same contexts: polling and waking happens in the same
///    thread, no `Send` is needed.
/// 2. Rust <-> Rust in different contexts: Futures cannot be sent between JS
///    contexts within the same Rust crate (because they are not `Send`).
/// 3. JS <-> Rust in different contexts: the `Promise` are [not transferable
///    between contexts][transferable], so this is impossible.
/// 4. JS <-> JS + Rust, this is possible, but safe since the Future is being
///    driven by JS in the same thread. If a Promise metaphor is desired, then
///    this must be built with JS talking to JS, because 3.
///
/// [webworker]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers
/// [jsfuture]: https://github.com/rustwasm/wasm-bindgen/blob/main/crates/futures/src/lib.rs
/// [transferable]: (https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects
#[cfg(target_arch = "wasm32")]
impl<T, F> UniffiCompatibleFuture<T> for F where F: Future<Output = T> {}

// === Public FFI API ===

/// Create a new [Handle] for a Rust future
///
/// For each exported async function, UniFFI will create a scaffolding function that uses this to
/// create the [Handle] to pass to the foreign code.
// Need to allow let_and_return, or clippy complains when the `ffi-trace` feature is disabled.
#[allow(clippy::let_and_return)]
pub fn rust_future_new<F, T, UT>(future: F, tag: UT) -> Handle
where
    // F is the future type returned by the exported async function.  It needs to be Send + `static
    // since it will move between threads for an indeterminate amount of time as the foreign
    // executor calls polls it and the Rust executor wakes it.  It does not need to by `Sync`,
    // since we synchronize all access to the values.
    F: UniffiCompatibleFuture<Result<T, LiftArgsError>> + 'static,
    // T is the output of the Future.  It needs to implement [LowerReturn].  Also it must be Send +
    // 'static for the same reason as F.
    T: LowerReturn<UT> + Send + 'static,
    // The UniFfiTag ZST. The Send + 'static bound is to keep rustc happy.
    UT: Send + 'static,
    // Needed to allocate a handle
    dyn RustFutureFfi<T::ReturnType>: HandleAlloc<UT>,
{
    let handle = HandleAlloc::new_handle(
        RustFuture::new(Box::pin(future), tag) as Arc<dyn RustFutureFfi<T::ReturnType>>
    );
    trace!("rust_future_new: {handle:?}");
    handle
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
    data: u64,
) where
    dyn RustFutureFfi<ReturnType>: HandleAlloc<UT>,
{
    trace!("rust_future_poll: {handle:?}");
    <dyn RustFutureFfi<ReturnType> as HandleAlloc<UT>>::get_arc(handle).ffi_poll(callback, data)
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
    dyn RustFutureFfi<ReturnType>: HandleAlloc<UT>,
{
    trace!("rust_future_cancel: {handle:?}");
    <dyn RustFutureFfi<ReturnType> as HandleAlloc<UT>>::get_arc(handle).ffi_cancel()
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
    dyn RustFutureFfi<ReturnType>: HandleAlloc<UT>,
{
    trace!("rust_future_complete: {handle:?}");
    <dyn RustFutureFfi<ReturnType> as HandleAlloc<UT>>::get_arc(handle).ffi_complete(out_status)
}

/// Free a Rust future, dropping the strong reference and releasing all references held by the
/// future.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_free<ReturnType, UT>(handle: Handle)
where
    dyn RustFutureFfi<ReturnType>: HandleAlloc<UT>,
{
    trace!("rust_future_free: {handle:?}");
    <dyn RustFutureFfi<ReturnType> as HandleAlloc<UT>>::consume_handle(handle).ffi_free()
}

// Derive HandleAlloc for dyn RustFutureFfi<T> for all FFI return types
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<u8>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<i8>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<u16>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<i16>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<u32>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<i32>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<u64>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<i64>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<f32>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<f64>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<*const std::ffi::c_void>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<crate::RustBuffer>);
derive_ffi_traits!(impl<UT> HandleAlloc<UT> for dyn RustFutureFfi<()>);
