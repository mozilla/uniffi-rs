# A handful of classes and functions to support the generated data structures.
# This would be a good candidate for isolating in its own ffi-support lib.

class InternalError(Exception):
    pass

class UniffiRustCallStatus(ctypes.Structure):
    """
    Error runtime.
    """
    _fields_ = [
        ("code", ctypes.c_int8),
        ("error_buf", UniffiRustBuffer),
    ]

    # These match the values from the uniffi::rustcalls module
    CALL_SUCCESS = 0
    CALL_ERROR = 1
    CALL_PANIC = 2

    def __str__(self):
        if self.code == UniffiRustCallStatus.CALL_SUCCESS:
            return "UniffiRustCallStatus(CALL_SUCCESS)"
        elif self.code == UniffiRustCallStatus.CALL_ERROR:
            return "UniffiRustCallStatus(CALL_ERROR)"
        elif self.code == UniffiRustCallStatus.CALL_PANIC:
            return "UniffiRustCallStatus(CALL_PANIC)"
        else:
            return "UniffiRustCallStatus(<invalid code>)"

def _rust_call(fn, *args):
    # Call a rust function
    return _rust_call_with_error(None, fn, *args)

def _rust_call_with_error(error_ffi_converter, fn, *args):
    # Call a rust function and handle any errors
    #
    # This function is used for rust calls that return Result<> and therefore can set the CALL_ERROR status code.
    # error_ffi_converter must be set to the UniffiConverter for the error class that corresponds to the result.
    call_status = UniffiRustCallStatus(code=UniffiRustCallStatus.CALL_SUCCESS, error_buf=UniffiRustBuffer(0, 0, None))

    args_with_error = args + (ctypes.byref(call_status),)
    result = fn(*args_with_error)
    uniffi_check_call_status(error_ffi_converter, call_status)
    return result

def uniffi_check_call_status(error_ffi_converter, call_status):
    if call_status.code == UniffiRustCallStatus.CALL_SUCCESS:
        pass
    elif call_status.code == UniffiRustCallStatus.CALL_ERROR:
        if error_ffi_converter is None:
            call_status.error_buf.free()
            raise InternalError("_rust_call_with_error: CALL_ERROR, but error_ffi_converter is None")
        else:
            raise error_ffi_converter.lift(call_status.error_buf)
    elif call_status.code == UniffiRustCallStatus.CALL_PANIC:
        # When the rust code sees a panic, it tries to construct a UniffiRustBuffer
        # with the message.  But if that code panics, then it just sends back
        # an empty buffer.
        if call_status.error_buf.len > 0:
            msg = UniffiConverterString.lift(call_status.error_buf)
        else:
            msg = "Unknown rust panic"
        raise InternalError(msg)
    else:
        raise InternalError("Invalid UniffiRustCallStatus code: {}".format(
            call_status.code))

# A function pointer for a callback as defined by UniFFI.
# Rust definition `fn(handle: u64, method: u32, args: UniffiRustBuffer, buf_ptr: *mut UniffiRustBuffer) -> int`
UNIFFI_FOREIGN_CALLBACK_T = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_ulonglong, ctypes.c_ulong, ctypes.POINTER(ctypes.c_char), ctypes.c_int, ctypes.POINTER(UniffiRustBuffer))

# UniFFI future continuation
UNIFFI_FUTURE_CONTINUATION_T = ctypes.CFUNCTYPE(None, ctypes.c_size_t, ctypes.c_int8)

