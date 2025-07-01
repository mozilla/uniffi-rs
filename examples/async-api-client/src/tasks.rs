/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

#[async_trait::async_trait]
#[uniffi::trait_interface]
pub trait TaskRunner: Send + Sync {
    async fn run_task(&self, task: Arc<dyn RustTask>);
}

#[uniffi::trait_interface]
pub trait RustTask: Send + Sync {
    fn execute(&self);
}

pub async fn run_task<F, T>(runner: &Arc<dyn TaskRunner>, closure: F) -> T
where
    F: FnOnce() -> T + Send + Sync + 'static,
    T: Send + 'static,
{
    let closure = Arc::new(TaskClosure::new(closure));
    runner
        .run_task(Arc::clone(&closure) as Arc<dyn RustTask>)
        .await;
    closure.take_result()
}

struct TaskClosure<F, T>
where
    F: FnOnce() -> T + Send + Sync,
    T: Send,
{
    inner: Mutex<TaskClosureInner<F, T>>,
}

enum TaskClosureInner<F, T>
where
    F: FnOnce() -> T + Send + Sync,
    T: Send,
{
    Pending(F),
    Running,
    Complete(T),
    Finished,
}

impl<F, T> TaskClosure<F, T>
where
    F: FnOnce() -> T + Send + Sync,
    T: Send,
{
    fn new(closure: F) -> Self {
        Self {
            inner: Mutex::new(TaskClosureInner::Pending(closure)),
        }
    }

    fn take_result(&self) -> T {
        let mut inner = self.inner.lock().unwrap();
        match *inner {
            TaskClosureInner::Pending(_) => panic!("Task never ran"),
            TaskClosureInner::Running => panic!("Task still running"),
            TaskClosureInner::Finished => panic!("Task already finished"),
            TaskClosureInner::Complete(_) => (),
        };
        match std::mem::replace(&mut *inner, TaskClosureInner::Finished) {
            TaskClosureInner::Complete(v) => v,
            _ => unreachable!(),
        }
    }
}

impl<F, T> RustTask for TaskClosure<F, T>
where
    F: FnOnce() -> T + Send + Sync,
    T: Send,
{
    fn execute(&self) {
        let mut inner = self.inner.lock().unwrap();
        match std::mem::replace(&mut *inner, TaskClosureInner::Running) {
            TaskClosureInner::Pending(f) => {
                let result = f();
                *inner = TaskClosureInner::Complete(result)
            }
            TaskClosureInner::Running => panic!("Task already started"),
            TaskClosureInner::Complete(_) => panic!("Task already executed"),
            TaskClosureInner::Finished => panic!("Task already finished"),
        }
    }
}
