use std::sync::Arc;
use pinky_swear::{Pinky, PinkySwear};

mod tasks;

pub trait PromiseInt: Send + Sync {
    fn resolve(&self, value: i32);
}

pub struct BindingsPromiseInt {
    pinky: Pinky<i32>,
}

impl BindingsPromiseInt {
    fn new() -> (PinkySwear<i32>, Self) {
        let (future, pinky) = PinkySwear::new();
        return (future, Self { pinky })
    }

    pub fn resolve(&self, value: i32) {
        self.pinky.swear(value);
        tasks::poll_tasks();
    }
}
pub trait AsyncCalculator: Send + Sync {
    fn double(&self, promise: Arc<BindingsPromiseInt>, value: i32);
    fn square(&self, promise: Arc<BindingsPromiseInt>, value: i32);
}

// Wrap the AsyncCalculator trait in a struct that's easier to use from rust
pub struct WrappedAsyncCalculator {
    inner: Box<dyn AsyncCalculator>,
}

impl WrappedAsyncCalculator {
    async fn double(&self, value: i32) -> i32 {
        let (future, promise) = BindingsPromiseInt::new();
        self.inner.double(Arc::new(promise), value);
        future.await
    }

    async fn square(&self, value: i32) -> i32 {
        let (future, promise) = BindingsPromiseInt::new();
        self.inner.square(Arc::new(promise), value);
        future.await
    }
}

pub fn double_then_square(promise: Box<dyn PromiseInt>, calculator: Box<dyn AsyncCalculator>, value: i32)
{
    tasks::Task::start(async move {
        let result = double_then_square_impl(
            WrappedAsyncCalculator { inner: calculator },
            value,
        ).await;
        promise.resolve(result);
    });
}

// Inner code that does the actual work
async fn double_then_square_impl(calculator: WrappedAsyncCalculator, value: i32) -> i32 {
    calculator.square(
        calculator.double(value).await
    ).await
}

uniffi_macros::include_scaffolding!("asynccalc");
