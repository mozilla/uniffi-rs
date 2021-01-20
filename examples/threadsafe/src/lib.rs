/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Simulation of a task doing something to keep a thread busy.
/// Up until now, everything has been synchronous and blocking, so
/// we can use this to simulate a CPU bound or I/O task.
fn busy_wait(ms: i32) {
    use std::{thread, time};
    let ms = ms as u64;
    let millis = time::Duration::from_millis(ms);
    thread::sleep(millis);
}

// This counter is a naive implementation of a counter that _should_ allow the
// waiting on one thread, and counting on another thread.
// It relies on uniffi's default locking mechanisms, provided by the
// `MutexHandleMap`, so doesn't actually work.
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

    fn busy_wait(&mut self, ms: i32) {
        self.is_busy = true;
        busy_wait(ms);
        self.is_busy = false;
    }

    fn increment_if_busy(&mut self) -> i32 {
        if self.is_busy {
            self.count += 1;
        }

        self.count
    }
}

// This counter is a better implementation of the counter above that allows
// waiting on one thread, and counting on another thread.
// It relies on its own locking mechanisms, and uses the `Threadsafe`
// attribute to tell uniffi to use the `ArcHandleMap`, so avoid the default
// locking strategy.
struct ThreadsafeCounter {
    is_busy: AtomicBool,
    count: AtomicI32,
}

// Interface structs labelled `Threadsafe` should (safely) implement
// `Send` and `Sync`.
static_assertions::assert_impl_all!(ThreadsafeCounter: Send, Sync);

impl ThreadsafeCounter {
    fn new() -> Self {
        Self {
            is_busy: AtomicBool::new(false),
            count: AtomicI32::new(0),
        }
    }

    fn busy_wait(&self, ms: i32) {
        self.is_busy.store(true, Ordering::SeqCst);
        busy_wait(ms);
        self.is_busy.store(false, Ordering::SeqCst);
    }

    fn increment_if_busy(&self) -> i32 {
        if self.is_busy.load(Ordering::SeqCst) {
            self.count.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.count.load(Ordering::SeqCst)
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/threadsafe.uniffi.rs"));
