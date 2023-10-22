// Magic numbers for the Rust proxy to call using the same mechanism as every other method.

private let IDX_CALLBACK_FREE: Int32 = 0
private let IDX_CALLBACK_CLONE: Int32 = 0x7FFF_FFFF;

// Callback return codes
private let UNIFFI_CALLBACK_SUCCESS: Int32 = 0
private let UNIFFI_CALLBACK_ERROR: Int32 = 1
private let UNIFFI_CALLBACK_UNEXPECTED_ERROR: Int32 = 2
