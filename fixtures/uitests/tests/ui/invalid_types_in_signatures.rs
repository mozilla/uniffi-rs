fn main() { /* empty main required by `trybuild` */}

#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum ErrorType {
    #[error("Foo")]
    Foo,
}

#[uniffi::export]
pub fn input_error(_e: ErrorType) { }

#[uniffi::export]
pub fn output_error() -> ErrorType {
    ErrorType::Foo
}

#[uniffi::export]
pub fn input_result(_r: Result<(), ErrorType>) { }

#[uniffi::export]
pub fn return_option_result() -> Option<Result<(), ErrorType>> {
    Some(Ok(()))
}

uniffi_macros::setup_scaffolding!();
