/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Defines the BlockingTaskQueue struct
//!
//! This module is responsible for the general handling of BlockingTaskQueue instances (cloning, droping, etc).
//! See `scheduler.rs` and the foreign bindings code for how the async functionality is implemented.

use super::scheduler::schedule_in_blocking_task_queue;
use std::num::NonZeroU64;

/// Foreign-managed blocking task queue that we can use to schedule futures
///
/// On the foreign side this is a Kotlin `CoroutineContext`, Python `Executor` or Swift
/// `DispatchQueue`. UniFFI converts those objects into this struct for the Rust code to use.
///
/// Rust async code can call [BlockingTaskQueue::execute] to run a closure in that
/// blocking task queue.  Use this for functions with blocking operations that should not be executed
/// in a normal async context.  Some examples are non-async file/network operations, long-running
/// CPU-bound tasks, blocking database operations, etc.
#[repr(C)]
pub struct BlockingTaskQueue {
    /// Opaque handle for the task queue
    pub handle: NonZeroU64,
    /// Method VTable
    ///
    /// This is simply a C struct where each field is a function pointer that inputs a
    /// BlockingTaskQueue handle
    pub vtable: &'static BlockingTaskQueueVTable,
}

#[repr(C)]
#[derive(Debug)]
pub struct BlockingTaskQueueVTable {
    clone: extern "C" fn(u64) -> u64,
    drop: extern "C" fn(u64),
}

// Note: see `scheduler.rs` for details on how BlockingTaskQueue is used.
impl BlockingTaskQueue {
    /// Run a closure in a blocking task queue
    pub async fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        schedule_in_blocking_task_queue(self.handle).await;
        f()
    }
}

impl Clone for BlockingTaskQueue {
    fn clone(&self) -> Self {
        let raw_handle = (self.vtable.clone)(self.handle.into());
        let handle = raw_handle
            .try_into()
            .expect("BlockingTaskQueue.clone() returned 0");
        Self {
            handle,
            vtable: self.vtable,
        }
    }
}

impl Drop for BlockingTaskQueue {
    fn drop(&mut self) {
        (self.vtable.drop)(self.handle.into())
    }
}
