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

use crate::{FfiDefault, Handle, LiftArgsError, LowerReturn, RustCallStatus};

/// Result code for [rust_future_poll].  This is passed to the continuation function.
#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RustFuturePoll {
    /// The future is ready and is waiting for [rust_future_complete] to be called
    Ready = 0,
    /// The future might be ready and [rust_future_poll] should be called again
    Wake = 1,
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
/// It is _not_ intended to be implemented by library users or bindings
/// implementers.
#[doc(hidden)]
#[cfg(not(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded")))]
pub trait UniffiCompatibleFuture<T>: Future<Output = T> + Send {}

#[doc(hidden)]
pub trait FutureLowerReturn<UT>: LowerReturn<UT> {}

/// The `Send` bound is required because the Foreign code may call the
/// `rust_future_*` methods from different threads.
#[cfg(not(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded")))]
impl<T, F> UniffiCompatibleFuture<T> for F where F: Future<Output = T> + Send {}
#[cfg(not(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded")))]
impl<UT, LR> FutureLowerReturn<UT> for LR where LR: LowerReturn<UT> + Send {}

/// `Future`'s on WASM32 are typically not `Send` because it's a single threaded environment.
///
/// Users can opt into allowing non-Send futures by using the `wasm-unstable-single-threaded`
/// feature.  This creates a `UniffiCompatibleFuture` impl for non-Send futures.
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
#[cfg(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded"))]
pub trait UniffiCompatibleFuture<T>: Future<Output = T> {}
#[cfg(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded"))]
impl<T, F> UniffiCompatibleFuture<T> for F where F: Future<Output = T> {}
#[cfg(all(target_arch = "wasm32", feature = "wasm-unstable-single-threaded"))]
impl<UT, LR> FutureLowerReturn<UT> for LR where LR: LowerReturn<UT> {}

// === Public FFI API ===

/// Create a new [Handle] for a Rust future
///
/// For each exported async function, UniFFI will create a scaffolding function that uses this to
/// create the [Handle] to pass to the foreign code.
// Need to allow let_and_return, or clippy complains when the `ffi-trace` feature is disabled.
#[allow(clippy::let_and_return)]
pub fn rust_future_new<F, T, UT>(future: F, tag: UT) -> Handle
where
    F: UniffiCompatibleFuture<Result<T, LiftArgsError>> + 'static,
    T: FutureLowerReturn<UT> + 'static,
{
    let rust_future = Arc::new(RustFuture::new(future, tag));
    let handle = Handle::from_arc(rust_future);
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
pub unsafe fn rust_future_poll<FfiType>(
    handle: Handle,
    callback: RustFutureContinuationCallback,
    data: u64,
) {
    trace!("rust_future_poll: {handle:?}");
    Handle::into_arc_borrowed::<RustFuture<FfiType>>(handle).poll(callback, data)
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
pub unsafe fn rust_future_cancel<FfiType>(handle: Handle) {
    trace!("rust_future_cancel: {handle:?}");
    Handle::into_arc_borrowed::<RustFuture<FfiType>>(handle).cancel()
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
pub unsafe fn rust_future_complete<FfiType>(
    handle: Handle,
    out_status: &mut RustCallStatus,
) -> FfiType
where
    FfiType: FfiDefault,
{
    trace!("rust_future_complete: {handle:?}");
    Handle::into_arc_borrowed::<RustFuture<FfiType>>(handle).complete(out_status)
}

/// Free a Rust future, dropping the strong reference and releasing all references held by the
/// future.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_free<FfiType>(handle: Handle) {
    trace!("rust_future_free: {handle:?}");
    Handle::into_arc_borrowed::<RustFuture<FfiType>>(handle).free()
}
