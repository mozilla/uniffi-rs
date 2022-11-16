// See `uniffi/src/ffi/rustfuture.rs` for documentation on these functions.

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn {{ ci.ffi_rustfuture_poll().name() }}(
    future: core::option::Option<&mut uniffi::RustFuture>,
    waker: core::option::Option<core::ptr::NonNull<uniffi::RustFutureForeignWaker>>,
    call_status: &mut uniffi::RustCallStatus
) -> bool {
    uniffi::ffi::uniffi_rustfuture_poll(future, waker, call_status)
}
