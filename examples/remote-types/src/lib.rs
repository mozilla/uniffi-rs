// It's often useful, but not required, to define type aliases for remote type.
//
// These control the names of the types in the generated code.
type AnyhowError = anyhow::Error;
type LogLevel = log::Level;

// Use #[uniffi::remote] to enable support for passing the types across the FFI

// For records/enums, wrap the item definition with `#[uniffi::remote]`.
// Copy each field/variant definitions exactly as they appear in the remote crate.
#[uniffi::remote(Enum)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

// For interfaces, wrap a unit struct with `#[uniffi::remote]`.
#[uniffi::remote(Object)]
pub struct AnyhowError;

// TODO:
//  - Add support for exporting methods on remote interfaces types
//  - Add support for Rust traits like `Display`

/// Logger that uses the remote types
#[derive(uniffi::Object)]
pub struct LogSink {}

#[uniffi::export]
impl LogSink {
    /// Construct a new LogSink that writes to a file
    /// We currently need to wrap AnyhowError in an Arc, but #2093 should fix that.
    #[uniffi::constructor]
    pub fn new(path: &str) -> Result<Self, std::sync::Arc<AnyhowError>> {
        // For this example, we don't actually do anything with the path except check that it's
        // non-empty.
        if path.is_empty() {
            Err(std::sync::Arc::new(AnyhowError::msg("Empty path")))
        } else {
            Ok(Self {})
        }
    }

    /// Log a message to our file
    pub fn log(&self, _level: log::Level, _msg: String) {
        // Pretend this code actually writes something out
    }
}

uniffi::setup_scaffolding!();
