/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tokio runtime management for UniFFI.
//!
//! This module provides a global Tokio runtime that can be configured once at startup
//! and automatically entered for all async functions marked with
//! `#[uniffi::export(async_runtime = "tokio")]`.

#[cfg(feature = "tokio")]
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};
#[cfg(feature = "tokio")]
use tokio::runtime::Runtime;

#[cfg(feature = "tokio")]
static TOKIO_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

/// Error returned when attempting to set the Tokio runtime more than once.
#[derive(Debug, Clone, Copy)]
pub struct AlreadySetError;

impl std::fmt::Display for AlreadySetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Tokio runtime has already been set")
    }
}

impl std::error::Error for AlreadySetError {}

/// Configure the global Tokio runtime.
///
/// This must be called **once** at application startup, before any async functions
/// annotated with `#[uniffi::export(async_runtime = "tokio")]` are called.
///
/// # Example
///
/// ```rust
/// use tokio::runtime::Builder;
/// use uniffi::set_tokio_runtime;
///
/// fn init() {
///     let rt = Builder::new_multi_thread()
///         .worker_threads(4)
///         .thread_name("my-app-tokio")
///         .build()
///         .unwrap();
///
///     set_tokio_runtime(rt).expect("runtime already set");
/// }
/// ```
#[cfg(feature = "tokio")]
pub fn set_tokio_runtime(rt: Runtime) -> Result<(), AlreadySetError> {
    TOKIO_RUNTIME.set(Arc::new(rt)).map_err(|_| AlreadySetError)
}

/// Get a reference to the configured Tokio runtime.
///
/// Returns `None` if `set_tokio_runtime` has not been called.
#[cfg(feature = "tokio")]
#[allow(dead_code)]
pub(crate) fn tokio_runtime() -> Option<Arc<Runtime>> {
    TOKIO_RUNTIME.get().cloned()
}

#[cfg(not(feature = "tokio"))]
pub(crate) fn tokio_runtime() -> Option<()> {
    None
}

/// A future wrapper that enters the shared Tokio runtime during polling.
///
/// This is similar to `async_compat::Compat` but uses the globally configured
/// Tokio runtime instead of a fallback.
#[cfg(feature = "tokio")]
pub struct TokioRuntimeCompat<F> {
    inner: F,
}

#[cfg(feature = "tokio")]
impl<F> TokioRuntimeCompat<F> {
    /// Wrap a future to run it in the shared Tokio runtime.
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "tokio")]
impl<F: Future> Future for TokioRuntimeCompat<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rt = TOKIO_RUNTIME
            .get()
            .expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup");
        let _guard = rt.enter();
        // SAFETY: We're not moving the inner future, just polling it
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.poll(cx)
    }
}

#[cfg(feature = "tokio")]
/// Wrap a future to run it in the shared Tokio runtime.
///
/// This is the primary way to use the shared Tokio runtime from generated code.
pub fn tokio_runtime_compat<F>(inner: F) -> TokioRuntimeCompat<F>
where
    F: Future,
{
    TokioRuntimeCompat::new(inner)
}
