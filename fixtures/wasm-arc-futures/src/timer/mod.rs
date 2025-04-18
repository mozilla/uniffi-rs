/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/
 */
use std::{future::Future, time::Duration};

/// TimerFuture extracted from fixtures/futures.
pub struct TimerFuture;
pub trait TimerService {
    type Future: Future;
    fn sleep(duration: Duration) -> Self::Future;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[allow(unused)]
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::*;
