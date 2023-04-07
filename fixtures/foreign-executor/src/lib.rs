/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use uniffi::ForeignExecutor;

pub struct ForeignExecutorTester {
    executor: ForeignExecutor,
    last_result: Arc<Mutex<Option<TestResult>>>,
}

// All constructor have to be defined in UDL for now
impl ForeignExecutorTester {
    fn new(executor: ForeignExecutor) -> Self {
        Self {
            executor,
            last_result: Arc::new(Mutex::new(None)),
        }
    }

    // Test inputting the ForeignExecutor from a Vec.  This tests that they can be written to a
    // `RustBuffer`
    fn new_from_sequence(executors: Vec<ForeignExecutor>) -> Self {
        assert_eq!(executors.len(), 1);
        Self::new(executors.into_iter().next().unwrap())
    }
}

#[uniffi::export]
impl ForeignExecutorTester {
    /// Schedule a fire-and-forget task to run the test
    fn schedule_test(&self, delay: u32) {
        let last_result = self.last_result.clone();
        *last_result.lock().unwrap() = None;
        // Start a thread to schedule the call.  This tests if the foreign bindings can handle
        // schedule callbacks from a thread that they don't manage.
        thread::scope(move |scope| {
            scope.spawn(move || {
                let start_time = Instant::now();
                let initial_thread_id = thread::current().id();
                // Schedule a call with the foreign executor
                self.executor.schedule(delay, move || {
                    // Return data on when/where the call happened.  We check that this matches the
                    // expectations in the foreign bindings tests
                    let call_happened_in_different_thread =
                        thread::current().id() != initial_thread_id;
                    let delay_ms = start_time.elapsed().as_millis() as u32;
                    *last_result.lock().unwrap() = Some(TestResult {
                        call_happened_in_different_thread,
                        delay_ms,
                    });
                });
            });
        });
    }

    fn get_last_result(&self) -> Option<TestResult> {
        self.last_result.lock().unwrap().take()
    }
}
#[derive(uniffi::Record)]
pub struct TestResult {
    pub call_happened_in_different_thread: bool,
    pub delay_ms: u32,
}

include!(concat!(env!("OUT_DIR"), "/foreign_executor.uniffi.rs"));
