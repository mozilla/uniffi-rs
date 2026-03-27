#[derive(uniffi::Error)]
pub enum Error {
}

// Can't use another Result as the E type
#[uniffi::export]
pub fn function(a: u32) -> Result<(), Result<u32, Error>> {
    todo!();
}
