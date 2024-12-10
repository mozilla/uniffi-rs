use std::sync::Arc;
use uniffi_one::{UniffiOneAsyncTrait, UniffiOneEnum, UniffiOneInterface, UniffiOneTrait};

#[derive(Default, uniffi::Record)]
pub struct SubLibType {
    pub maybe_enum: Option<UniffiOneEnum>,
    pub maybe_trait: Option<Arc<dyn UniffiOneTrait>>,
    pub maybe_interface: Option<Arc<UniffiOneInterface>>,
}

#[uniffi::export]
fn get_sub_type(existing: Option<SubLibType>) -> SubLibType {
    existing.unwrap_or_default()
}

struct OneImpl;

impl UniffiOneTrait for OneImpl {
    fn hello(&self) -> String {
        "sub-lib trait impl says hello".to_string()
    }
}

#[async_trait::async_trait]
impl UniffiOneAsyncTrait for OneImpl {
    async fn hello_async(&self) -> String {
        "sub-lib async trait impl says hello".to_string()
    }
}

#[uniffi::export]
fn get_trait_impl() -> Arc<dyn UniffiOneTrait> {
    Arc::new(OneImpl {})
}

#[uniffi::export]
fn get_async_trait_impl() -> Arc<dyn UniffiOneAsyncTrait> {
    Arc::new(OneImpl {})
}

#[derive(uniffi::Object)]
struct UniffiOneTraitWrapper {
    inner: Arc<dyn UniffiOneTrait>,
}

#[uniffi::export]
impl UniffiOneTraitWrapper {
    #[uniffi::constructor]
    fn new(inner: Arc<dyn UniffiOneTrait>) -> Self {
        Self { inner }
    }

    fn hello(&self) -> String {
        self.inner.hello()
    }
}

#[derive(uniffi::Object)]
struct UniffiOneAsyncTraitWrapper {
    inner: Arc<dyn UniffiOneAsyncTrait>,
}

#[uniffi::export]
impl UniffiOneAsyncTraitWrapper {
    #[uniffi::constructor]
    fn new(inner: Arc<dyn UniffiOneAsyncTrait>) -> Self {
        Self { inner }
    }

    async fn hello_async(&self) -> String {
        self.inner.hello_async().await
    }
}

uniffi::setup_scaffolding!("imported_types_sublib");
