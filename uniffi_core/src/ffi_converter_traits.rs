/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use crate::{
    try_lift_from_rust_buffer, FfiDefault, MetadataBuffer, Result, RustBuffer, RustCallStatus,
    UnexpectedUniFFICallbackError,
};

/// Trait defining how to transfer values via the FFI layer.
///
/// The `FfiConverter` trait defines how to pass values of a particular type back-and-forth over
/// the uniffi generated FFI layer, both as standalone argument or return values, and as
/// part of serialized compound data structures. `FfiConverter` is mainly used in generated code.
/// The goal is to make it easy for the code generator to name the correct FFI-related function for
/// a given type.
///
/// FfiConverter has a generic parameter, that's filled in with a type local to the UniFFI consumer crate.
/// This allows us to work around the Rust orphan rules for remote types. See
/// `https://mozilla.github.io/uniffi-rs/internals/lifting_and_lowering.html#code-generation-and-the-fficonverter-trait`
/// for details.
///
/// ## Scope
///
/// It could be argued that FfiConverter handles too many concerns and should be split into
/// separate traits (like `FfiLiftAndLower`, `FfiSerialize`, `FfiReturn`).  However, there are good
/// reasons to keep all the functionality in one trait.
///
/// The main reason is that splitting the traits requires fairly complex blanket implementations,
/// for example `impl<UT, T> FfiReturn<UT> for T: FfiLiftAndLower<UT>`.  Writing these impls often
/// means fighting with `rustc` over what counts as a conflicting implementation.  In fact, as of
/// Rust 1.66, that previous example conflicts with `impl<UT> FfiReturn<UT> for ()`, since other
/// crates can implement `impl FfiReturn<MyLocalType> for ()`. In general, this path gets
/// complicated very quickly and that distracts from the logic that we want to define, which is
/// fairly simple.
///
/// The main downside of having a single `FfiConverter` trait is that we need to implement it for
/// types that only support some of the functionality.  For example, `Result<>` supports returning
/// values, but not lifting/lowering/serializing them.  This is a bit weird, but works okay --
/// especially since `FfiConverter` is rarely used outside of generated code.
///
/// ## Safety
///
/// This is an unsafe trait (implementing it requires `unsafe impl`) because we can't guarantee
/// that it's safe to pass your type out to foreign-language code and back again. Buggy
/// implementations of this trait might violate some assumptions made by the generated code,
/// or might not match with the corresponding code in the generated foreign-language bindings.
///
/// In general, you should not need to implement this trait by hand, and should instead rely on
/// implementations generated from your component UDL via the `uniffi-bindgen scaffolding` command.
pub unsafe trait FfiConverter<UT>: Sized {
    /// The low-level type used for passing values of this type over the FFI.
    ///
    /// This must be a C-compatible type (e.g. a numeric primitive, a `#[repr(C)]` struct) into
    /// which values of the target rust type can be converted.
    ///
    /// For complex data types, we currently recommend using `RustBuffer` and serializing
    /// the data for transfer. In theory it could be possible to build a matching
    /// `#[repr(C)]` struct for a complex data type and pass that instead, but explicit
    /// serialization is simpler and safer as a starting point.
    type FfiType;

    /// The type that should be returned by scaffolding functions for this type.
    ///
    /// This is usually the same as `FfiType`, but `Result<>` has specialized handling.
    type ReturnType: FfiDefault;

    /// The `FutureCallback<T>` type used for async functions
    ///
    /// This is almost always `FutureCallback<Self::ReturnType>`.  The one exception is the
    /// unit type, see that `FfiConverter` impl for details.
    type FutureCallback: Copy;

    /// Lower a rust value of the target type, into an FFI value of type Self::FfiType.
    ///
    /// This trait method is used for sending data from rust to the foreign language code,
    /// by (hopefully cheaply!) converting it into something that can be passed over the FFI
    /// and reconstructed on the other side.
    ///
    /// Note that this method takes an owned value; this allows it to transfer ownership in turn to
    /// the foreign language code, e.g. by boxing the value and passing a pointer.
    fn lower(obj: Self) -> Self::FfiType;

    /// Lower this value for scaffolding function return
    ///
    /// This method converts values into the `Result<>` type that [rust_call] expects. For
    /// successful calls, return `Ok(lower_return)`.  For errors that should be translated into
    /// thrown exceptions on the foreign code, serialize the error into a RustBuffer and return
    /// `Err(buf)`
    fn lower_return(obj: Self) -> Result<Self::ReturnType, RustBuffer>;

    /// Lift a rust value of the target type, from an FFI value of type Self::FfiType.
    ///
    /// This trait method is used for receiving data from the foreign language code in rust,
    /// by (hopefully cheaply!) converting it from a low-level FFI value of type Self::FfiType
    /// into a high-level rust value of the target type.
    ///
    /// Since we cannot statically guarantee that the foreign-language code will send valid
    /// values of type Self::FfiType, this method is fallible.
    fn try_lift(v: Self::FfiType) -> Result<Self>;

    /// Lift a Rust value for a callback interface method result
    fn lift_callback_return(buf: RustBuffer) -> Self {
        try_lift_from_rust_buffer(buf).expect("Error reading callback interface result")
    }

    /// Lift a Rust value for a callback interface method error result
    ///
    /// This is called for "expected errors" -- the callback method returns a Result<> type and the
    /// foreign code throws an exception that corresponds to the error type.
    fn lift_callback_error(_buf: RustBuffer) -> Self {
        panic!("Callback interface method returned unexpected error")
    }

    /// Lift a Rust value for an unexpected callback interface error
    ///
    /// The main reason this is called is when the callback interface throws an error type that
    /// doesn't match the Rust trait definition.  It's also called for corner cases, like when the
    /// foreign code doesn't follow the FFI contract.
    ///
    /// The default implementation panics unconditionally.  Errors used in callback interfaces
    /// handle this using the `From<UnexpectedUniFFICallbackError>` impl that the library author
    /// must provide.
    fn handle_callback_unexpected_error(e: UnexpectedUniFFICallbackError) -> Self {
        panic!("Callback interface failure: {e}")
    }

    /// Write a rust value into a buffer, to send over the FFI in serialized form.
    ///
    /// This trait method can be used for sending data from rust to the foreign language code,
    /// in cases where we're not able to use a special-purpose FFI type and must fall back to
    /// sending serialized bytes.
    ///
    /// Note that this method takes an owned value because it's transferring ownership
    /// to the foreign language code via the RustBuffer.
    fn write(obj: Self, buf: &mut Vec<u8>);

    /// Read a rust value from a buffer, received over the FFI in serialized form.
    ///
    /// This trait method can be used for receiving data from the foreign language code in rust,
    /// in cases where we're not able to use a special-purpose FFI type and must fall back to
    /// receiving serialized bytes.
    ///
    /// Since we cannot statically guarantee that the foreign-language code will send valid
    /// serialized bytes for the target type, this method is fallible.
    ///
    /// Note the slightly unusual type here - we want a mutable reference to a slice of bytes,
    /// because we want to be able to advance the start of the slice after reading an item
    /// from it (but will not mutate the actual contents of the slice).
    fn try_read(buf: &mut &[u8]) -> Result<Self>;

    /// Invoke a `FutureCallback` to complete a async call
    fn invoke_future_callback(
        callback: Self::FutureCallback,
        callback_data: *const (),
        return_value: Self::ReturnType,
        call_status: RustCallStatus,
    );

    /// Type ID metadata, serialized into a [MetadataBuffer]
    const TYPE_ID_META: MetadataBuffer;
}

/// FfiConverter for Arc-types
///
/// This trait gets around the orphan rule limitations, which prevent library crates from
/// implementing `FfiConverter` on an Arc. When this is implemented for T, we generate an
/// `FfiConverter` impl for Arc<T>.
///
/// Note: There's no need for `FfiConverterBox`, since Box is a fundamental type.
///
/// ## Safety
///
/// This has the same safety considerations as FfiConverter
pub unsafe trait FfiConverterArc<UT>: Send + Sync {
    type FfiType;
    type ReturnType: FfiDefault;
    type FutureCallback: Copy;

    fn lower(obj: Arc<Self>) -> Self::FfiType;
    fn lower_return(obj: Arc<Self>) -> Result<Self::ReturnType, RustBuffer>;
    fn try_lift(v: Self::FfiType) -> Result<Arc<Self>>;
    fn lift_callback_return(buf: RustBuffer) -> Arc<Self> {
        try_lift_from_rust_buffer(buf).expect("Error reading callback interface result")
    }
    fn lift_callback_error(_buf: RustBuffer) -> Arc<Self> {
        panic!("Callback interface method returned unexpected error")
    }
    fn handle_callback_unexpected_error(e: UnexpectedUniFFICallbackError) -> Arc<Self> {
        panic!("Callback interface failure: {e}")
    }
    fn write(obj: Arc<Self>, buf: &mut Vec<u8>);
    fn try_read(buf: &mut &[u8]) -> Result<Arc<Self>>;
    fn invoke_future_callback(
        callback: Self::FutureCallback,
        callback_data: *const (),
        return_value: Self::ReturnType,
        call_status: RustCallStatus,
    );
    const TYPE_ID_META: MetadataBuffer;
}

unsafe impl<T, UT> FfiConverter<UT> for Arc<T>
where
    T: FfiConverterArc<UT> + ?Sized,
{
    type FfiType = T::FfiType;
    type ReturnType = T::ReturnType;
    type FutureCallback = T::FutureCallback;

    fn lower(obj: Self) -> Self::FfiType {
        T::lower(obj)
    }

    fn lower_return(obj: Self) -> Result<Self::ReturnType, RustBuffer> {
        T::lower_return(obj)
    }

    fn try_lift(v: Self::FfiType) -> Result<Self> {
        T::try_lift(v)
    }

    fn lift_callback_return(buf: RustBuffer) -> Self {
        T::lift_callback_error(buf)
    }

    fn lift_callback_error(buf: RustBuffer) -> Self {
        T::lift_callback_error(buf)
    }

    fn handle_callback_unexpected_error(e: UnexpectedUniFFICallbackError) -> Self {
        T::handle_callback_unexpected_error(e)
    }

    fn write(obj: Self, buf: &mut Vec<u8>) {
        T::write(obj, buf)
    }

    fn try_read(buf: &mut &[u8]) -> Result<Self> {
        T::try_read(buf)
    }

    fn invoke_future_callback(
        callback: Self::FutureCallback,
        callback_data: *const (),
        return_value: Self::ReturnType,
        call_status: RustCallStatus,
    ) {
        T::invoke_future_callback(callback, callback_data, return_value, call_status)
    }

    const TYPE_ID_META: MetadataBuffer = T::TYPE_ID_META;
}
