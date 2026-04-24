/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Functionality for caching JNI classes, method ids, etc.

use std::ffi::CStr;
use std::sync::OnceLock;

use jni_sys::*;

/// Caches a JNI class
pub struct CachedClass {
    once_lock: OnceLock<jobject>,
    class_name: &'static CStr,
}

impl CachedClass {
    pub const fn new(class_name: &'static CStr) -> Self {
        Self {
            once_lock: OnceLock::new(),
            class_name,
        }
    }

    /// Get the class object
    ///
    /// # Safety
    ///
    /// env must point to a valid JNIEnv
    pub unsafe fn get(&self, env: *mut JNIEnv) -> jclass {
        // Safety we're using the JNI API correctly
        unsafe {
            *self.once_lock.get_or_init(|| {
                let class = ((**env).v1_2.FindClass)(env, self.class_name.as_ptr());
                if class.is_null() {
                    panic!("Class not found: {}", self.class_name.to_string_lossy());
                }
                ((**env).v1_2.NewGlobalRef)(env, class)
            })
        }
    }
}

/// Caches a JNI class and method ID for a method call
pub struct CachedMethod {
    once_lock: OnceLock<jmethodID>,
    class_name: &'static CStr,
    method_name: &'static CStr,
    sig: &'static CStr,
}

impl CachedMethod {
    pub const fn new(
        class_name: &'static CStr,
        method_name: &'static CStr,
        sig: &'static CStr,
    ) -> Self {
        Self {
            once_lock: OnceLock::new(),
            class_name,
            method_name,
            sig,
        }
    }

    /// Get the method id
    ///
    /// # Safety
    ///
    /// env must point to a valid JNIEnv
    pub unsafe fn get(&self, env: *mut JNIEnv) -> jmethodID {
        // Safety we're using the JNI API correctly
        unsafe {
            *self.once_lock.get_or_init(|| {
                let class = ((**env).v1_2.FindClass)(env, self.class_name.as_ptr());
                if class.is_null() {
                    panic!("Class not found: {}", self.class_name.to_string_lossy());
                }
                let method_id = ((**env).v1_2.GetMethodID)(
                    env,
                    class,
                    self.method_name.as_ptr(),
                    self.sig.as_ptr(),
                );
                if method_id.is_null() {
                    panic!(
                        "Method not found: {} {}",
                        self.class_name.to_string_lossy(),
                        self.method_name.to_string_lossy(),
                    );
                }
                let class = ((**env).v1_2.NewGlobalRef)(env, class);
                if class.is_null() {
                    panic!(
                        "Failed to create global ref: {} {}",
                        self.class_name.to_string_lossy(),
                        self.method_name.to_string_lossy(),
                    );
                }
                method_id
            })
        }
    }

    /// Call a method that returns void
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return void and the args must match the signature
    pub unsafe fn call_void<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<(), jthrowable> {
        let method_id = self.get(env);
        ((**env).v1_2.CallVoidMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(())
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns an object
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return an object and the args must match the signature
    pub unsafe fn call_object<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jobject, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallObjectMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a boolean
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a boolean and the args must match the signature
    pub unsafe fn call_boolean<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jboolean, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallBooleanMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a byte
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a byte and the args must match the signature
    pub unsafe fn call_byte<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jbyte, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallByteMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a int
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return an int and the args must match the signature
    pub unsafe fn call_int<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jint, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallIntMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a long
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a long and the args must match the signature
    pub unsafe fn call_long<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jlong, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallLongMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a float
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a float and the args must match the signature
    pub unsafe fn call_float<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jfloat, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallFloatMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a method that returns a double
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a double and the args must match the signature
    pub unsafe fn call_double<const N: usize>(
        &self,
        env: *mut JNIEnv,
        obj: jobject,
        args: [jvalue; N],
    ) -> Result<jdouble, jthrowable> {
        let method_id = self.get(env);
        let v = ((**env).v1_2.CallDoubleMethodA)(env, obj, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }
}

/// Caches the JNI class and static method ID for a static method call
pub struct CachedStaticMethod {
    once_lock: OnceLock<(jclass, jmethodID)>,
    class_name: &'static CStr,
    method_name: &'static CStr,
    sig: &'static CStr,
}

impl CachedStaticMethod {
    pub const fn new(
        class_name: &'static CStr,
        method_name: &'static CStr,
        sig: &'static CStr,
    ) -> Self {
        Self {
            once_lock: OnceLock::new(),
            class_name,
            method_name,
            sig,
        }
    }

    /// Get the class and method id
    ///
    /// # Safety
    ///
    /// env must point to a valid JNIEnv
    pub unsafe fn get(&self, env: *mut JNIEnv) -> (jclass, jmethodID) {
        // Safety we're using the JNI API correctly
        unsafe {
            *self.once_lock.get_or_init(|| {
                let class = ((**env).v1_2.FindClass)(env, self.class_name.as_ptr());
                if class.is_null() {
                    panic!("Class not found: {}", self.class_name.to_string_lossy());
                }
                let method_id = ((**env).v1_2.GetStaticMethodID)(
                    env,
                    class,
                    self.method_name.as_ptr(),
                    self.sig.as_ptr(),
                );
                if method_id.is_null() {
                    panic!(
                        "Static method not found: {} {}",
                        self.class_name.to_string_lossy(),
                        self.method_name.to_string_lossy(),
                    );
                }
                let class = ((**env).v1_2.NewGlobalRef)(env, class);
                if class.is_null() {
                    panic!(
                        "Failed to create global ref: {} {}",
                        self.class_name.to_string_lossy(),
                        self.method_name.to_string_lossy(),
                    );
                }
                (class, method_id)
            })
        }
    }

    /// Call a static method that returns void
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return void and the args must match the signature
    pub unsafe fn call_void<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<(), jthrowable> {
        let (class, method_id) = self.get(env);
        ((**env).v1_2.CallStaticVoidMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(())
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns an object
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return an object and the args must match the signature
    pub unsafe fn call_object<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jobject, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticObjectMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a boolean
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a boolean and the args must match the signature
    pub unsafe fn call_boolean<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jboolean, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticBooleanMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a byte
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a byte and the args must match the signature
    pub unsafe fn call_byte<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jbyte, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticByteMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a int
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return an int and the args must match the signature
    pub unsafe fn call_int<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jint, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticIntMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a long
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a long and the args must match the signature
    pub unsafe fn call_long<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jlong, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticLongMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a float
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a float and the args must match the signature
    pub unsafe fn call_float<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jfloat, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticFloatMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }

    /// Call a static method that returns a double
    ///
    /// # Safety
    ///
    /// * env must point to a valid JNIEnv
    /// * The method must return a double and the args must match the signature
    pub unsafe fn call_double<const N: usize>(
        &self,
        env: *mut JNIEnv,
        args: [jvalue; N],
    ) -> Result<jdouble, jthrowable> {
        let (class, method_id) = self.get(env);
        let v = ((**env).v1_2.CallStaticDoubleMethodA)(env, class, method_id, args.as_ptr());
        let exc = ((**env).v1_2.ExceptionOccurred)(env);
        if exc.is_null() {
            Ok(v)
        } else {
            Err(exc)
        }
    }
}

/// Safety: we can share these between threads since we store global references
unsafe impl std::marker::Sync for CachedClass {}

/// Safety: we can share these between threads since we store global references
unsafe impl std::marker::Sync for CachedMethod {}

/// Safety: we can share these between threads since we store global references
unsafe impl std::marker::Sync for CachedStaticMethod {}
