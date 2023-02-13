// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/uitests/src/errors.udl");

fn main() { /* empty main required by `trybuild` */}

#[derive(Debug)]
enum ArithmeticError {
  IntegerOverflow,
  // Since this is listed in the UDL as not having fields and is used in a callback interface, it
  // really needs to have no fields.
  DivisionByZero { numerator: u64 },
  // Tuple-style fields are also invalid
  UnexpectedError(String),
}

impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for ArithmeticError {
    fn from(e: uniffi::UnexpectedUniFFICallbackError) -> ArithmeticError {
        ArithmeticError::UnexpectedError(e.to_string())
    }
}

pub trait Calculator {
    fn add(&self, a: u64, b: u64) -> Result<u64, ArithmeticError>;
}
