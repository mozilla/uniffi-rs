fun {{ error_type.throw_error_fn_kt() }}(ffiBuffer: Long) {
    val uniffiReader = uniffi.FfiBufferCursor(ffiBuffer)
    throw {{ error_type.read_fn_kt }}(uniffiReader)
}
