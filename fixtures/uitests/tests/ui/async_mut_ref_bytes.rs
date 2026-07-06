fn main() {} /* empty main required by `trybuild` */

// `&mut [u8]` is not allowed in an async exported function: Rust could resume
// on a background thread and write to the buffer after the foreign caller has
// continued and possibly freed or moved it.
#[uniffi::export]
pub async fn bad_async_mut_bytes(buf: &mut [u8]) {
    buf[0] = 1;
}

uniffi_macros::setup_scaffolding!();
