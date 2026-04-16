use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

pub struct UniffiOneType {
    pub sval: String,
}

pub enum UniffiOneEnum {
    One,
    Two,
}

#[derive(uniffi::Record)]
pub struct UniffiOneProcMacroType {
    pub sval: String,
}

#[derive(uniffi::Record)]
pub struct BindingRenamedType {
    pub value: String,
}

#[derive(Default)]
pub struct UniffiOneInterface {
    current: AtomicI32,
}

impl UniffiOneInterface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self) -> i32 {
        self.current.fetch_add(1, Ordering::Relaxed) + 1
    }
}

// A Three Letter Acronym, testing unusual capitalization.
#[derive(Eq, PartialEq, Debug, uniffi::Record)]
pub struct UniffiOneTLA {}

#[uniffi::export]
fn get_my_proc_macro_type(t: UniffiOneProcMacroType) -> UniffiOneProcMacroType {
    t
}

#[uniffi::export]
async fn get_uniffi_one_async() -> UniffiOneEnum {
    UniffiOneEnum::One
}

#[uniffi::export(with_foreign)]
pub trait UniffiOneTrait: Send + Sync {
    fn hello(&self) -> String;
}

// A couple of errors used as external types.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum UniffiOneError {
    #[error("{0}")]
    Oops(String),
}

#[derive(Debug, uniffi::Object, thiserror::Error)]
#[uniffi::export(Debug, Display)]
pub struct UniffiOneErrorInterface {
    pub e: String,
}

#[uniffi::export]
impl UniffiOneErrorInterface {
    fn message(&self) -> String {
        self.e.clone()
    }
}

impl std::fmt::Display for UniffiOneErrorInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UniffiOneErrorInterface({})", self.e)
    }
}

// We *must* have this so error support is generated. We should fix that and remove this. See #2393.
#[uniffi::export]
fn _just_to_get_error_support() -> Result<(), UniffiOneErrorInterface> {
    Ok(())
}

// Note `UDL` vs `Udl` is important here to test foreign binding name fixups.
#[uniffi::trait_interface]
pub trait UniffiOneUDLTrait: Send + Sync {
    fn hello(&self) -> String;
}

#[derive(uniffi::Record, Default)]
pub struct UniffiOneRecordContainingInterface {
    #[uniffi(default)]
    pub inner: Arc<UniffiOneInterface>,
}

uniffi::include_scaffolding!("uniffi-one");
