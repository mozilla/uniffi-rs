pub type AnyhowError = anyhow::Error;
pub type LogLevel = log::Level;

#[uniffi::remote(Object)]
pub struct AnyhowError;

#[uniffi::remote(Enum)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}
