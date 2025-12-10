/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::ComplexError;
use uniffi::{ffi_buffer_size, FfiSerialize, LiftReturn, Lower, RustBuffer, RustCallStatus};

/// Test the pointer FFI version of our scaffolding functions by manually calling one.
///
/// We use the `get_complex_error` version, since it's one of more complex cases:
///    - It inputs multiple arguments
///    - The Rust function returns a Result<> type, which means the pointer-ffi scaffolding function
///      needs to deserialize the `RustCallStatus` out pointer, pass it to the regular scaffolding
///      function, and everything needs to be put back together in the end.
#[test]
fn test_ffi_buffer_scaffolding() {
    // Call the pointer-ffi version of the scaffolding function for `divide_by_text`
    //
    // This simulates the work that happens on the foreign side.
    fn call_pointer_ffi_divide_by_text(
        value: f32,
        value_as_text: String,
    ) -> Result<f32, ComplexError> {
        // Create a buffer big enough to store the arguments/return value
        let mut ffi_buffer = [0_u8; ffi_buffer_size!((f32, RustBuffer), (f32, RustCallStatus))];

        // Lower the arguments
        let value_lowered = <f32 as Lower<crate::UniFfiTag>>::lower(value);
        let value_as_text_lowered = <String as Lower<crate::UniFfiTag>>::lower(value_as_text);
        // Serialize the lowered arguments plus the RustCallStatus into the argument buffer
        let args_cursor = &mut ffi_buffer.as_mut_slice();
        unsafe {
            <f32 as FfiSerialize>::write(args_cursor, value_lowered);
            <RustBuffer as FfiSerialize>::write(args_cursor, value_as_text_lowered);
            // Call the pointer-ffi version of the scaffolding function
            crate::uniffi_ptr_uniffi_coverall_fn_func_divide_by_text(ffi_buffer.as_mut_ptr());

            // Deserialize the return and the RustCallStatus from the return buffer
            let return_cursor = &mut ffi_buffer.as_slice();
            let rust_call_status = <RustCallStatus as FfiSerialize>::read(return_cursor);
            let return_value = <f32 as FfiSerialize>::read(return_cursor);
            // Lift the return from the deserialized value.
            <Result<f32, ComplexError> as LiftReturn<crate::UniFfiTag>>::lift_foreign_return(
                return_value,
                rust_call_status,
            )
        }
    }

    assert_eq!(call_pointer_ffi_divide_by_text(1.0, "2".into()), Ok(0.5));
    assert_eq!(call_pointer_ffi_divide_by_text(5.0, "2.5".into()), Ok(2.0));
    assert_eq!(
        call_pointer_ffi_divide_by_text(1.0, "two".into()),
        Err(ComplexError::UnknownError)
    );
}
