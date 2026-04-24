const UNIFFI_RUST_FUTURE_POLL_AGAIN: i32 = 0;
const UNIFFI_RUST_FUTURE_CANCELLED: i32 = 1;
const UNIFFI_RUST_FUTURE_COMPLETE: i32 = 2;
const UNIFFI_RUST_FUTURE_ERROR: i32 = 3;
const UNIFFI_RUST_FUTURE_FAILED: i32 = 4;

const UNIFFI_KOTLIN_FUTURE_OK: i32 = 0;
const UNIFFI_KOTLIN_FUTURE_ERR: i32 = 1;

/// Stores a future and scheduler for a Kotlin -> Rust call
///
/// The future should either write to the FFI buffer it inputted and return
/// `UNIFFI_RUST_FUTURE_COMPLETE` or return `UNIFFI_RUST_FUTURE_FAILED`
struct UniffiRustFuture {
    scheduler: ::std::sync::Mutex<uniffi::Scheduler<UniffiRustFutureContinutation>>,
    future: ::std::sync::Mutex<::std::pin::Pin<::std::boxed::Box<dyn std::future::Future<Output = i32> + ::std::marker::Send>>>,
}

impl UniffiRustFuture {
    fn new(future: impl ::std::future::Future<Output = i32> + std::marker::Send + 'static) -> ::std::sync::Arc<Self> {
        ::std::sync::Arc::new(Self {
            scheduler: ::std::sync::Mutex::new(uniffi::Scheduler::new()),
            future: ::std::sync::Mutex::new(::std::boxed::Box::pin(future)),
        })
    }

    fn into_handle(self: ::std::sync::Arc<Self>) -> i64 {
        ::std::sync::Arc::into_raw(self).expose_provenance() as i64
    }
}

impl ::std::task::Wake for UniffiRustFuture {
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
        // Safety:
        //
        // * uniffi_get_global_jvm() returns a valid JavaVM pointer
        // * The args match the function signature
        unsafe {
            uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |env| {
                if UNIFFI_CONTINUATION_RESUME.call_void(env, [
                    uniffi_jni::jvalue {
                        l: self.continuation,
                    },
                ]).is_err() {
                    ((**env).v1_2.ExceptionClear)(env);
                    eprintln!("Exception calling uniffi.uniffiContinuationResume");

                }
                ((**env).v1_2.DeleteGlobalRef)(env, self.continuation);
            });
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_uniffiRustFuturePoll(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
    continuation: uniffi_jni::jobject,
) -> i32 {
    uniffi::trace!("RustFuture::free: {uniffi_future_handle:x}");
    unsafe {
        uniffi_jni::rust_call(uniffi_env, |uniffi_env| {
            // Safety:
            // We assume the Kotlin side of the FFI sent us a future handle
            let uniffi_future: ::std::sync::Arc::<UniffiRustFuture> = unsafe {
                // Increment the strong count since we're creating a new `Arc`.
                let ptr = ::std::ptr::with_exposed_provenance::<UniffiRustFuture>(uniffi_future_handle as usize);
                ::std::sync::Arc::increment_strong_count(ptr);
                ::std::sync::Arc::from_raw(ptr)
            };

            if uniffi_future.scheduler.lock().unwrap().is_cancelled() {
                uniffi::trace!("RustFuture::poll: cancelled");
                return Ok(UNIFFI_RUST_FUTURE_CANCELLED);
            }

            let mut locked = uniffi_future.future.lock().unwrap();
            let waker = ::std::task::Waker::from(::std::sync::Arc::clone(&uniffi_future));
            let pinned: std::pin::Pin<&mut dyn ::std::future::Future<Output = i32>> = locked.as_mut();
            match pinned.poll(&mut ::std::task::Context::from_waker(&waker)) {
                ::std::task::Poll::Ready(code) => {
                    uniffi::trace!("RustFuture::poll: ready {code:x}");
                    Ok(code)
                }
                ::std::task::Poll::Pending => {
                    let continuation = UniffiRustFutureContinutation {
                        continuation: ((**uniffi_env).v1_2.NewGlobalRef)(uniffi_env, continuation),
                    };
                    uniffi_future.scheduler.lock().unwrap().store(continuation);
                    Ok(UNIFFI_RUST_FUTURE_POLL_AGAIN)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_uniffiRustFutureCancel(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
) {
    uniffi::trace!("RustFuture::cancel: {uniffi_future_handle:x}");
    // Safety:
    // We assume the Kotlin side of the FFI sent us a future handle
    let ptr = unsafe {
        ::std::ptr::with_exposed_provenance::<UniffiRustFuture>(uniffi_future_handle as usize)
    };
    (*ptr).scheduler.lock().unwrap().cancel();
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_uniffiRustFutureFree(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
) {
    uniffi::trace!("RustFuture::free: {uniffi_future_handle:x}");
    // Safety:
    // We assume the Kotlin side of the FFI sent us a future handle
    unsafe {
        let ptr = ::std::ptr::with_exposed_provenance::<UniffiRustFuture>(uniffi_future_handle as usize);
        ::std::sync::Arc::decrement_strong_count(ptr);
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_uniffiKotlinFutureComplete(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_kotlin_future_handle: i64,
    uniffi_kotlin_future_result: i32,
) {
    uniffi::trace!("KotlinFuture::complete: {uniffi_kotlin_future_handle:x} {uniffi_kotlin_future_result}");
    // Safety:
    // We assume the Kotlin side of the FFI sent us a valid future handle
    let sender = unsafe {
        uniffi::oneshot::Sender::from_raw(
            ::std::ptr::with_exposed_provenance::<_>(uniffi_kotlin_future_handle as usize)
        )
    };
    sender.send(uniffi_kotlin_future_result)
}
