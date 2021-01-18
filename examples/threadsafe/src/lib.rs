/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use static_assertions;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Simulation of a task doing something to keep a thread busy.
/// Up until now, everything has been synchronous and blocking, so
/// we can use this to simulate a CPU bound or I/O task.
fn busy_wait(cycles: i32) -> i32 {
    let mut res = 1;
    for _ in 0..cycles {
        // Dumb as rocks divergent function that the compiler _probably_
        // can't optimize away.
        res = res * -1;
    }
    res
}

struct Counter {
    is_busy: bool,
    count: i32,
}

impl Counter {
    fn new() -> Self {
        Self {
            is_busy: false,
            count: 0,
        }
    }

    fn busy_wait(&mut self, cycles: i32) -> i32 {
        self.is_busy = true;
        let res = busy_wait(cycles);
        self.is_busy = false;
        res
    }

    fn increment_if_busy(&mut self) -> i32 {
        if self.is_busy {
            self.count = self.count + 1;
        }

        self.count
    }
}

struct ThreadsafeCounter {
    is_busy: AtomicBool,
    count: AtomicI32,
}

static_assertions::assert_impl_all!(ThreadsafeCounter: Send, Sync);

impl ThreadsafeCounter {
    fn new() -> Self {
        Self {
            is_busy: AtomicBool::new(false),
            count: AtomicI32::new(0),
        }
    }

    fn busy_wait(&self, cycles: i32) -> i32 {
        self.is_busy.store(true, Ordering::SeqCst);
        let res = busy_wait(cycles);
        self.is_busy.store(false, Ordering::SeqCst);
        res
    }

    fn increment_if_busy(&self) -> i32 {
        let current = if self.is_busy.load(Ordering::SeqCst) {
            self.count.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.count.load(Ordering::SeqCst)
        };

        current
    }
}

include!(concat!(env!("OUT_DIR"), "/threadsafe.uniffi.rs"));
