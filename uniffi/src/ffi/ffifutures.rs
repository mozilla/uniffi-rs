use std::{future::Future, pin::Pin};

#[repr(transparent)]
pub struct FfiFuture<T>(Pin<Box<dyn Future<Output = T> + Send + 'static>>)
where
    T: Send + 'static;

impl<T> FfiFuture<T>
where
    T: Send + 'static,
{
    pub fn new(f: impl Future<Output = T> + Send + 'static) -> Self {
        Self(Box::pin(f))
    }
}

impl<T> super::FfiDefault for FfiFuture<T>
where
    T: Send + 'static + Default,
{
    fn ffi_default() -> Self {
        FfiFuture::new(core::future::ready(T::default()))
    }
}
