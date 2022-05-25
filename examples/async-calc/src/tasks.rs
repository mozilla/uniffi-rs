use futures::task::{waker_ref, ArcWake};
use futures::future::{BoxFuture, FutureExt};
use std::cell::RefCell;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task::Context;

// Extremely simple async task executer system
//
// - Use Task::start() to start a new async task.  It should be passed an Unit-returning async function.
// - Tasks should only await pinky-swear promises
// - After you resolve a pinky swear promise, call `poll_tasks()` to run any tasks that were
//   awaiting on that promise.

// Top-level task object.
pub struct Task(Mutex<BoxFuture<'static, ()>>);

// Tasks that are ready to be polled.
//
// I think thread local is good for some corner cases, but maybe we don't need it.
thread_local! {
  static TASKS_TO_POLL: RefCell<Vec<Arc<Task>>> = RefCell::new(Vec::new());
}

impl Task {
    pub fn start(future: impl Future<Output=()> + 'static + Send) {
        Self::poll(Arc::new(Self(Mutex::new(future.boxed()))))
    }

    #[allow(unused_must_use)]
    pub fn poll(arc_self: Arc<Self>) {
        let waker = waker_ref(&arc_self);
        let context = &mut Context::from_waker(&waker);
        arc_self.0.lock().unwrap().as_mut().poll(context);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        TASKS_TO_POLL.with(|tasks_to_poll| tasks_to_poll.borrow_mut().push(arc_self.clone()));
    }
}

pub fn poll_tasks() {
  TASKS_TO_POLL.with(|tasks_to_poll| {
      for task in tasks_to_poll.borrow_mut().drain(..) {
          Task::poll(task);
      }
  });
}
