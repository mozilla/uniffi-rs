/// Invalid result type, but it requires going through `use` statements and type aliases.
#[uniffi::export]
pub fn function(a: u32) -> submod::Result<String> {
    todo!();
}

pub mod submod {
    use std::io::Error;

    pub type Result<T> = Result2<T>;

    pub type Result2<T, E=Error> = std::result::Result<T, E>;
}
