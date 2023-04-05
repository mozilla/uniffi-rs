#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Badness has happened")]
    AllGoneWrong,
}

pub type Result<T> = std::result::Result<T, MyError>;

pub fn returns_unary_result_alias() -> Result<()> {
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/unary-result-alias.uniffi.rs"));
