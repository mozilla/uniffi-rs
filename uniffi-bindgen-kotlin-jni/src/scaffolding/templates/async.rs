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

{%- for rust_result in root.rust_async_callable_results() %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ rust_result.async_poll_fn() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
    continuation: uniffi_jni::jobject,
    {%- if rust_result.return_type.is_some() %}
    completion: uniffi_jni::jobject
    {%- endif %}
) -> i32 {
    uniffi::trace!("RustFuture::poll: {uniffi_future_handle:x}");
    {%- if let Some(return_type) = rust_result.return_type %}
    static UNIFFI_COMPLETE_METHOD: uniffi_jni::CachedMethod = uniffi_jni::CachedMethod::new(
        c"uniffi/{{ rust_result.async_complete_class() }}",
        c"complete",
        c"({% for ffi_type in return_type.ffi_types %}{{ ffi_type.jni_signature() }}{% endfor %})V",
    );
    {%- endif %}
    unsafe {
        uniffi_jni::rust_call_with_env(uniffi_env, |uniffi_env| {
            // Safety:
            // We assume the Kotlin side of the FFI sent us a future handle
            let uniffi_future: ::std::sync::Arc::<UniffiRustFuture<{{ rust_result.return_type_rs() }}>> = unsafe {
                // Increment the strong count since we're creating a new `Arc`.
                let ptr = ::std::ptr::with_exposed_provenance::<UniffiRustFuture<{{ rust_result.return_type_rs() }}>>(uniffi_future_handle as usize);
                ::std::sync::Arc::increment_strong_count(ptr);
                ::std::sync::Arc::from_raw(ptr)
            };

            if uniffi_future.scheduler.lock().unwrap().is_cancelled() {
                uniffi::trace!("RustFuture::poll: cancelled");
                return Ok(UNIFFI_RUST_FUTURE_CANCELLED);
            }

            let mut locked = uniffi_future.future.lock().unwrap();
            let waker = ::std::task::Waker::from(::std::sync::Arc::clone(&uniffi_future));
            let pinned: std::pin::Pin<&mut dyn ::std::future::Future<Output = {{ rust_result.return_type_rs() }}>> = locked.as_mut();
            match pinned.poll(&mut ::std::task::Context::from_waker(&waker)) {
                ::std::task::Poll::Ready(uniffi_return) => {
                    uniffi::trace!("RustFuture::poll: ready");

                    {%- if let Some(throws_type) = rust_result.throws_type %}
                    let uniffi_return = match uniffi_return {
                        Ok(v) => v,
                        Err(uniffi_err) => {
                            let uniffi_err_lowered = {{ throws_type.lower_fn_rs() }}(uniffi_env, uniffi_err)?;
                            let uniffi_err_obj = {{ throws_type.lift_kt_from_rust_var() }}.call_object(
                                uniffi_env,
                                [
                                    {%- for (var, ffi_type) in throws_type.ffi_values_rs("uniffi_err_lowered") %}
                                    uniffi_jni::jvalue {
                                        {{ ffi_type.jvalue_field() }}: {{ var }},
                                    },
                                    {%- endfor %}
                                ]
                            ).to_anyhow_result(uniffi_env, "{{ throws_type.lift_fn_kt() }}")?;
                            if ((**uniffi_env).v1_4.Throw)(uniffi_env, uniffi_err_obj) != 0 {
                                uniffi::deps::anyhow::bail!("Failed to throw exception for {{ rust_result.async_poll_fn() }}");
                            }
                            // The return value doesn't matter, since the Kotlin code will throw once it's
                            // resumes.  Let's use UNIFFI_RUST_FUTURE_CANCELLED so that if that fails somehow
                            // we the async function will still fail.
                            return Ok(UNIFFI_RUST_FUTURE_CANCELLED);
                        }
                    };
                    {%- endif %}

                    {%- if let Some(return_type) = rust_result.return_type %}
                    let uniffi_return_lowered = {{ return_type.lower_fn_rs() }}(uniffi_env, uniffi_return)?;
                    UNIFFI_COMPLETE_METHOD.call_void(
                        uniffi_env,
                        completion,
                        [
                            {%- for (var, ffi_type) in return_type.ffi_values_rs("uniffi_return_lowered") %}
                            uniffi_jni::jvalue {
                                {{ ffi_type.jvalue_field() }}: {{ var }},
                            },
                            {%- endfor %}
                        ],
                    ).to_anyhow_result(uniffi_env, "{{ rust_result.async_complete_class() }}.complete")?;
                    {%- endif %}

                    Ok(UNIFFI_RUST_FUTURE_COMPLETE)
                }
                ::std::task::Poll::Pending => {
                    let continuation = UniffiRustFutureContinutation {
                        continuation: ((**uniffi_env).v1_2.NewGlobalRef)(uniffi_env, continuation),
                    };
                    uniffi_future.scheduler.lock().unwrap().store(continuation);
                    Ok(UNIFFI_RUST_FUTURE_PENDING)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ rust_result.async_cancel_fn() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
) {
    uniffi::trace!("RustFuture::cancel: {uniffi_future_handle:x}");
    // Safety:
    // We assume the Kotlin side of the FFI sent us a future handle
    unsafe {
        let ptr = ::std::ptr::with_exposed_provenance::<UniffiRustFuture<{{ rust_result.return_type_rs() }}>>(uniffi_future_handle as usize);
        (*ptr).scheduler.lock().unwrap().cancel();
    };
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ rust_result.async_free_fn() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_future_handle: i64,
) {
    uniffi::trace!("RustFuture::free: {uniffi_future_handle:x}");
    // Safety:
    // We assume the Kotlin side of the FFI sent us a future handle
    unsafe {
        let ptr = ::std::ptr::with_exposed_provenance::<UniffiRustFuture<{{ rust_result.return_type_rs() }}>>(uniffi_future_handle as usize);
        ::std::sync::Arc::decrement_strong_count(ptr);
    };
}
{%- endfor %}

{%- for callback_result in root.kotlin_async_callable_results() %}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ callback_result.async_complete_success_fn() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    future_handle: i64,
    {%- if let Some(return_type) = callback_result.return_type %}
    {%- for ffi_type in return_type.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
    {%- endif %}
) {
    uniffi::trace!("{{ callback_result.async_complete_success_fn() }}: {future_handle:x}");
    unsafe {
        let sender = uniffi::oneshot::Sender::<uniffi::Result<{{ callback_result.return_type_rs() }}>>::from_raw(
            ::std::ptr::with_exposed_provenance::<_>(future_handle as usize)
        );
        // Use `rust_call` rather than `rust_call_with_env`.  This way unexpected errors get sent
        // via the oneshot channel rather than turning into Kotlin exceptions.
        let return_result = uniffi_jni::rust_call(move || {
            {%- if let Some(return_type) = callback_result.return_type %}
            let uniffi_return = {{ return_type.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_type in return_type.ffi_types %}
                v{{ loop.index0 }},
                {%- endfor %}
            )?;
            {%- else %}
            let uniffi_return = ();
            {%- endif %}
            {%- if callback_result.throws_type.is_some() %}
            uniffi::Result::Ok(::std::result::Result::Ok(uniffi_return))
            {%- else %}
            uniffi::Result::Ok(uniffi_return)
            {%- endif %}
        });
        sender.send(return_result.map_err(|msg| uniffi::deps::anyhow::anyhow!("{msg}")));
    }
}

{%- if let Some(throws_type) = callback_result.throws_type %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ callback_result.async_complete_error_fn() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    future_handle: i64,
    {%- for ffi_type in throws_type.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) {
    uniffi::trace!("{{ callback_result.async_complete_error_fn() }}: {future_handle:x}");
    unsafe {
        let sender = uniffi::oneshot::Sender::<uniffi::Result<{{ callback_result.return_type_rs() }}>>::from_raw(
            ::std::ptr::with_exposed_provenance::<_>(future_handle as usize)
        );
        // Use `rust_call` rather than `rust_call_with_env`.  This way unexpected errors get sent
        // via the oneshot channel rather than turning into Kotlin exceptions.
        let return_result = uniffi_jni::rust_call(|| {
            let err = {{ throws_type.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_type in throws_type.ffi_types %}
                v{{ loop.index0 }},
                {%- endfor %}
            )?;
            uniffi::Result::Ok(::std::result::Result::Err(err))
        });
        sender.send(return_result.map_err(|msg| uniffi::deps::anyhow::anyhow!("{msg}")));
    }
}
{%- endif %}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ callback_result.async_complete_unexpected_error_fn() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    future_handle: i64,
) {
    uniffi::trace!("{{ callback_result.async_complete_unexpected_error_fn() }}: {future_handle:x}");
    // Safety:
    // * We assume the Kotlin side sent us valid future handles
    let sender = unsafe {
        uniffi::oneshot::Sender::<uniffi::Result<{{ callback_result.return_type_rs() }}>>::from_raw(
            ::std::ptr::with_exposed_provenance::<_>(future_handle as usize)
        )
    };
    sender.send(uniffi::Result::Err(uniffi::deps::anyhow::anyhow!("Unexpected callback error")));
}
{%- endfor %}
