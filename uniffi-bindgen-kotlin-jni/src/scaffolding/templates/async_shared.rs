const UNIFFI_RUST_FUTURE_PENDING: i32 = 0;
const UNIFFI_RUST_FUTURE_CANCELLED: i32 = 1;
const UNIFFI_RUST_FUTURE_COMPLETE: i32 = 2;

/// Stores a future and scheduler for a Kotlin -> Rust call
struct UniffiRustFuture<T> {
    scheduler: ::std::sync::Mutex<uniffi::Scheduler<UniffiRustFutureContinutation>>,
    future: ::std::sync::Mutex<::std::pin::Pin<::std::boxed::Box<dyn std::future::Future<Output = T> + ::std::marker::Send>>>,
}

impl<T> UniffiRustFuture<T> {
    fn new(future: impl ::std::future::Future<Output = T> + std::marker::Send + 'static) -> ::std::sync::Arc<Self> {
        ::std::sync::Arc::new(Self {
            scheduler: ::std::sync::Mutex::new(uniffi::Scheduler::new()),
            future: ::std::sync::Mutex::new(::std::boxed::Box::pin(future)),
        })
    }

    fn into_handle(self: ::std::sync::Arc<Self>) -> i64 {
        ::std::sync::Arc::into_raw(self).expose_provenance() as i64
    }
}

impl<T> ::std::task::Wake for UniffiRustFuture<T> {
    fn wake(self: ::std::sync::Arc<Self>) {
        self.scheduler.lock().unwrap().wake();
    }

    fn wake_by_ref(self: &::std::sync::Arc<Self>) {
        self.scheduler.lock().unwrap().wake();
    }
}

struct UniffiRustFutureContinutation {
    continuation: uniffi_jni::jobject,
}

// Safety:
// It's safe to pass `jobject` pointers to another thread
unsafe impl ::std::marker::Send for UniffiRustFutureContinutation {}

impl uniffi::RustFutureCallback for UniffiRustFutureContinutation {
    fn invoke(self, _poll: uniffi::RustFuturePoll) {
        // Note: we ignore the `poll` value.  These bindings don't use it, instead they check the
        // return value of the poll function to know when the future is ready.
        static UNIFFI_CONTINUATION_RESUME: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
            c"uniffi/UniffiKt",
            c"uniffiContinuationResume",
            c"(Lkotlin/coroutines/Continuation;)V",
        );
        unsafe {
            uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |env| {
                UNIFFI_CONTINUATION_RESUME.call_void(env, [
                    uniffi_jni::jvalue {
                        l: self.continuation,
                    },
                ]).warn_on_exception(env, "uniffiContinuationResume");
                ((**env).v1_2.DeleteGlobalRef)(env, self.continuation);
            });
        }
    }
}
