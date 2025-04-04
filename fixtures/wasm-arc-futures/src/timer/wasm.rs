/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/
 */
use std::time::Duration;

use gloo_timers::future::TimeoutFuture;

use super::{TimerFuture, TimerService};

impl TimerService for TimerFuture {
    type Future = TimeoutFuture;
    fn sleep(duration: Duration) -> Self::Future {
        let millis = duration.as_millis();
        let millis = millis.try_into().unwrap();
        TimeoutFuture::new(millis)
    }
}
