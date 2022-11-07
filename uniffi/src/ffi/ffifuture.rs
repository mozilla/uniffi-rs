use super::FfiDefault;
use std::{
    future::{self, Future},
    pin::Pin,
};

#[repr(transparent)]
pub struct FfiFuture<T>(Pin<Box<dyn Future<Output = T> + Send + 'static>>)
where
    T: Send + 'static;

impl<T> FfiFuture<T>
where
    T: Send + 'static,
{
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self(Box::pin(future))
    }
}

impl<T> FfiDefault for FfiFuture<T>
where
    T: Send + 'static + FfiDefault,
{
    fn ffi_default() -> Self {
        Self::new(future::ready(T::ffi_default()))
    }
}
