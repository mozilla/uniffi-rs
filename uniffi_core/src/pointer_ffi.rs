/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Pointer FFI versions of builtin FFI functions

use std::{slice, sync::Arc};

use crate::{
    ffi::rustfuture::RustFuture, ffi_buffer_size, FfiDefault, FfiSerialize, FutureLowerReturn,
    Handle, LiftArgsError, RustBuffer, RustCallStatus, RustFutureContinuationCallback,
    UniffiCompatibleFuture,
};

/// This helper allocates a new byte buffer owned by the Rust code, and returns it
/// to the foreign-language code as a `RustBuffer` struct. Callers must eventually
/// free the resulting buffer, either by explicitly calling [`uniffi_rustbuffer_free`] defined
/// below, or by passing ownership of the buffer back into Rust code.
///
/// # Safety
///
/// The `ffi_buffer` argument must be serialized according to the Pointer FFI protocol.
pub unsafe fn rustbuffer_alloc(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((u64)));
    let size = <u64 as FfiSerialize>::read(&mut args_buf);

    let rust_buffer = RustBuffer::new_with_size(size);
    let mut uniffi_return_buf =
        ::std::slice::from_raw_parts_mut(ffi_buffer, ffi_buffer_size!((RustBuffer)));
    trace!("FFI rustbuffer_alloc {rust_buffer:?}");
    RustBuffer::write(&mut uniffi_return_buf, rust_buffer);
}

/// Free a byte buffer that had previously been passed to the foreign language code.
///
/// # Safety
///
/// The `ffi_buffer` argument must be serialized according to the Pointer FFI protocol.
pub unsafe fn rustbuffer_free(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((RustBuffer)));
    let rust_buffer = <RustBuffer as FfiSerialize>::read(&mut args_buf);
    trace!("FFI rustbuffer_free {rust_buffer:?}");
    rust_buffer.destroy();
}

/// RustFuture wrapped for the pointer FFI
///
/// The main reason for this trait is that it has no generic types.  This allows us to use a single
/// `rust_future_complete` complete function, regardless of the FfiType.
trait RustFuturePointerFfi {
    /// Complete a RustFuture using the pointer FFI
    ///
    /// # Safety
    ///
    /// The `ffi_buffer` argument must have enough space to hold both the return value and call
    /// status.
    unsafe fn complete(&self, ffi_buffer: *mut u8);

    // These are just forwarded to the `RustFuture` methods.  We need to do this in order to access
    // them from `dyn RustFuturePointerFfi`
    fn poll(self: Arc<Self>, callback: RustFutureContinuationCallback, data: u64);
    fn cancel(&self);
    fn free(&self);
}

impl<FfiType> RustFuturePointerFfi for RustFuture<FfiType>
where
    FfiType: FfiSerialize + FfiDefault,
{
    fn poll(self: Arc<Self>, callback: RustFutureContinuationCallback, data: u64) {
        RustFuture::poll(self, callback, data);
    }

    unsafe fn complete(&self, ffi_buffer: *mut u8) {
        let mut call_status = RustCallStatus::default();
        let return_value = RustFuture::complete(self, &mut call_status);
        let mut return_buf =
            slice::from_raw_parts_mut(ffi_buffer, ffi_buffer_size!((RustCallStatus, FfiType)));
        RustCallStatus::write(&mut return_buf, call_status);
        FfiType::write(&mut return_buf, return_value);
    }

    fn cancel(&self) {
        RustFuture::cancel(self);
    }

    fn free(&self) {
        RustFuture::free(self);
    }
}

/// Create a new [Handle] for a Rust future
///
/// For each exported async function, UniFFI will create a scaffolding function that uses this to
/// create the [Handle] to pass to the foreign code.
///
// Need to allow let_and_return, or clippy complains when the `ffi-trace` feature is disabled.
//
/// # Safety
///
/// The `ffi_buffer` argument must have enough space to handle the returned handle
#[allow(clippy::let_and_return)]
pub unsafe fn rust_future_new<F, T, UT>(future: F, tag: UT, ffi_buffer: *mut u8)
where
    F: UniffiCompatibleFuture<Result<T, LiftArgsError>> + 'static,
    T: FutureLowerReturn<UT> + 'static,
    T::ReturnType: FfiSerialize,
{
    // Use the double Arc trick to create a sized inner value, which can be turned into a handle.
    let rust_future: Arc<Arc<dyn RustFuturePointerFfi>> =
        Arc::new(Arc::new(RustFuture::new(future, tag)));
    let handle = Handle::from_arc(rust_future);
    trace!("rust_future_new(ptr): {handle:?}");
    let mut return_buf = slice::from_raw_parts_mut(ffi_buffer, ffi_buffer_size!((Handle)));
    Handle::write(&mut return_buf, handle);
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
pub unsafe fn rust_future_poll(ffi_buffer: *mut u8, callback: RustFutureContinuationCallback) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((Handle, u64)));
    let handle = Handle::read(&mut args_buf);
    let data = u64::read(&mut args_buf);

    #[cfg(feature = "ffi-trace")]
    let raw_handle = handle.as_raw();
    trace!("rust_future_poll(ptr): {raw_handle:x}");
    let future: Arc<Arc<dyn RustFuturePointerFfi>> = Handle::into_arc_borrowed(handle);
    let f: Arc<dyn RustFuturePointerFfi> = (*future).clone();
    f.poll(callback, data);
    trace!("rust_future_poll(ptr) returning: {raw_handle:x}");
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
pub unsafe fn rust_future_cancel(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((Handle, u64)));
    let handle = Handle::read(&mut args_buf);
    trace!("rust_future_cancel(ptr): {handle:?}");
    let future: Arc<Arc<dyn RustFuturePointerFfi>> =
        Handle::into_arc_borrowed::<Arc<dyn RustFuturePointerFfi>>(handle);
    future.cancel()
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
pub unsafe fn rust_future_complete(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((Handle, u64)));
    let handle = Handle::read(&mut args_buf);
    trace!("rust_future_cancel(ptr): {handle:?}");
    let future: Arc<Arc<dyn RustFuturePointerFfi>> =
        Handle::into_arc_borrowed::<Arc<dyn RustFuturePointerFfi>>(handle);
    future.complete(ffi_buffer);
}

/// Free a Rust future, dropping the strong reference and releasing all references held by the
/// future.
///
/// # Safety
///
/// The [Handle] must not previously have been passed to [rust_future_free]
pub unsafe fn rust_future_free(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((Handle, u64)));
    let handle = Handle::read(&mut args_buf);
    trace!("rust_future_cancel(ptr): {handle:?}");
    let future: Arc<Arc<dyn RustFuturePointerFfi>> =
        Handle::into_arc_borrowed::<Arc<dyn RustFuturePointerFfi>>(handle);
    (*future).clone().free()
}
