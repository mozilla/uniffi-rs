/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Pointer FFI versions of builtin FFI functions

use std::{mem, slice, sync::Arc};

use crate::{
    ffi::rustfuture::RustFuture, ffi_buffer_size, oneshot, FfiDefault, FfiSerialize,
    FutureLowerReturn, Handle, LiftArgsError, RustBuffer, RustCallStatus, RustFutureCallback,
    RustFuturePoll, UniffiCompatibleFuture,
};

/// Pointer FFI callback function
pub type CallbackFn = unsafe extern "C" fn(ffi_buffer: *mut u8);

/// Callback function alongside an opaque data value
///
/// This is the typically way callbacks are handled in the pointer FFI.
#[derive(Debug)]
pub struct BoundCallbackFn {
    pub callback: CallbackFn,
    pub data: Handle,
}

impl RustFutureCallback for CallbackFn {
    fn invoke(self, data: u64, poll: RustFuturePoll) {
        let mut ffi_buffer = [0_u8; ffi_buffer_size!((u64, i8))];
        let args_buf = &mut ffi_buffer.as_mut_slice();
        // Safety: we serialize the buffer according to the pointer FFI protocol.
        unsafe {
            <u64 as FfiSerialize>::write(args_buf, data);
            <i8 as FfiSerialize>::write(args_buf, poll as i8);
            self(ffi_buffer.as_mut_ptr());
        }
    }
}

impl FfiSerialize for CallbackFn {
    const SIZE: usize = 8;

    unsafe fn get(buf: &[u8]) -> Self {
        let buf: &[u64] = mem::transmute(buf);
        // Casting a `u64` to a function pointer requires casting it to a data pointer, then
        // use transmute to get a function pointer.
        let ptr = buf[0] as *const ();
        mem::transmute(ptr)
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        let buf: &mut [u64] = mem::transmute(buf);
        // Casting a `u64` to a function pointer requires casting it to a `usize`, then to a `u64`
        buf[0] = value as usize as u64;
    }
}

impl FfiSerialize for BoundCallbackFn {
    const SIZE: usize = 16;

    unsafe fn get(buf: &[u8]) -> Self {
        let callback = CallbackFn::get(buf);
        let data = Handle::get(&buf[8..]);
        BoundCallbackFn { callback, data }
    }

    unsafe fn put(buf: &mut [u8], value: Self) {
        CallbackFn::put(buf, value.callback);
        Handle::put(&mut buf[8..], value.data);
    }
}

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
    fn poll(self: Arc<Self>, callback: CallbackFn, data: u64);
    fn cancel(&self);
    fn free(&self);
}

impl<FfiType> RustFuturePointerFfi for RustFuture<FfiType, CallbackFn>
where
    FfiType: FfiSerialize + FfiDefault,
{
    fn poll(self: Arc<Self>, callback: CallbackFn, data: u64) {
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
pub unsafe fn rust_future_poll(ffi_buffer: *mut u8) {
    let mut args_buf =
        slice::from_raw_parts(ffi_buffer, ffi_buffer_size!((Handle, BoundCallbackFn)));
    let handle = Handle::read(&mut args_buf);
    let callback = BoundCallbackFn::read(&mut args_buf);

    #[cfg(feature = "ffi-trace")]
    let raw_handle = handle.as_raw();
    trace!("rust_future_poll(ptr): {raw_handle:x}");
    let future: Arc<Arc<dyn RustFuturePointerFfi>> = Handle::into_arc_borrowed(handle);
    let f: Arc<dyn RustFuturePointerFfi> = (*future).clone();
    f.poll(callback.callback, callback.data.as_raw());
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

/// Perform the initial work for calling a foreign async method
///
/// Returns a BoundCallbackFn to send to the foreign code and a oneshot receiver to await in the
/// Rust code
pub fn start_foreign_future() -> (BoundCallbackFn, oneshot::Receiver<ForeignFutureFfiBuffer>) {
    let (sender, receiver) = oneshot::channel::<ForeignFutureFfiBuffer>();
    let callback = BoundCallbackFn {
        callback: foreign_future_complete,
        data: Handle::from_pointer(sender.into_raw()),
    };

    (callback, receiver)
}

/// Foreign future FFI buffer
///
/// This is just a regular buffer wrapped in a newtype so that we can pass it over a oneshot
/// channel.
pub struct ForeignFutureFfiBuffer {
    pub buf: *const u8,
}

unsafe impl Send for ForeignFutureFfiBuffer {}
unsafe impl Sync for ForeignFutureFfiBuffer {}

/// # Safety
///
/// The `ffi_buffer` argument must be serialized according to the Pointer FFI protocol.
pub unsafe extern "C" fn foreign_future_complete(ffi_buffer: *mut u8) {
    let mut args_buf = slice::from_raw_parts(ffi_buffer, u64::SIZE);
    let sender_raw = u64::read(&mut args_buf);
    let channel: oneshot::Sender<ForeignFutureFfiBuffer> =
        unsafe { oneshot::Sender::from_raw(sender_raw as *mut ()) };
    channel.send(ForeignFutureFfiBuffer {
        buf: args_buf.as_ptr(),
    });
}

/// Stores the foreign future dropped callback and calls it once this is dropped.
///
/// This should be stored in the generated async function.  It's drop method will ensure that the
/// foreign callback is called when the future is called.
pub struct ForeignFutureDroppedCallbackContainer {
    callback: BoundCallbackFn,
    #[allow(unused)] // only used in `drop()`
    trait_name: &'static str,
    #[allow(unused)] // only used in `drop()`
    method_name: &'static str,
}

impl ForeignFutureDroppedCallbackContainer {
    /// # Safety
    ///
    /// `ffi_buffer` must be contain a `u64` value to send to the callback
    pub unsafe fn new(
        callback: BoundCallbackFn,
        trait_name: &'static str,
        method_name: &'static str,
    ) -> Self {
        trace!(
            "Creating foreign future dropped callback ({}::{} - {:?})",
            trait_name,
            method_name,
            callback,
        );
        Self {
            callback,
            trait_name,
            method_name,
        }
    }
}

impl Drop for ForeignFutureDroppedCallbackContainer {
    fn drop(&mut self) {
        trace!(
            "Calling foreign future dropped callback ({}::{} - {:?})",
            self.trait_name,
            self.method_name,
            self.callback
        );
        let mut buf = [0u8; u64::SIZE];
        // Safety: were calling the dropped callback according to the pointer FFI protocol.
        unsafe {
            Handle::write(&mut buf.as_mut_slice(), mem::take(&mut self.callback.data));
            (self.callback.callback)(buf.as_mut_ptr());
        }
    }
}
