/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ffi_support::{ExternError, Handle, HandleError, HandleMap, IntoFfi};
use std::sync::{Arc, RwLock};

/// `ArcHandleMap` is a relatively thin wrapper around `RwLock<HandleMap<Arc<T>>>`.
/// This is only suitable for objects that implement `Sync` and `Send` and is
/// for objects that are able to look after their own locking, and need to be called
/// from multiple foreign language threads at the same time.
///
/// In contrast, the `ConcurrentHandleMap` is aliased as `MutexHandleMap` to show the difference:
/// objects with `mut` methods can be used, but the objects can only be accessed from one thread
/// at a time.
///
/// The `Threadsafe` annotation is used to choose which handle map `uniffi` uses.
///
/// This module also provides the `UniffiMethodCall` trait, which allows generated scaffolding
/// to switch almost seemlessly.
// Some care is taken to protect that handlemap itself being read and written concurrently, and
// that the lock is held for the least amount of time; however, if it is ever poisoned, it will
// panic on both read and write.
pub struct ArcHandleMap<T>
where
    T: Sync + Send,
{
    /// The underlying map. Public so that more advanced use-cases
    /// may use it as they please.
    pub map: RwLock<HandleMap<Arc<T>>>,
}

impl<T: Sync + Send> ArcHandleMap<T> {
    /// Construct a new `ArcHandleMap`.
    pub fn new() -> Self {
        ArcHandleMap {
            map: RwLock::new(HandleMap::new()),
        }
    }

    /// Get the number of entries in the `ArcHandleMap`.
    ///
    /// This takes the map's `read` lock.
    #[inline]
    pub fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }

    /// Returns true if the `ArcHandleMap` is empty.
    ///
    /// This takes the map's `read` lock.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert an item into the map, returning the newly allocated handle to the
    /// item.
    ///
    /// This takes the map's `write` lock which the object is inserted.
    pub fn insert(&self, v: T) -> Handle {
        let mut map = self.map.write().unwrap();
        map.insert(Arc::new(v))
    }

    /// Remove an item from the map.
    ///
    /// This takes the map's `write` lock while removing from the map, but
    /// not while the object is being dropped.
    pub fn delete(&self, h: Handle) -> Result<(), HandleError> {
        // We use `remove` and not delete (and use the inner block) to ensure
        // that if `v`'s destructor panics, we aren't holding the write lock
        // when it happens, so that the map itself doesn't get poisoned.
        let v = {
            let mut map = self.map.write().unwrap();
            map.remove(h)
        };
        v.map(drop)
    }

    /// Convenient wrapper for `delete` which takes a `u64` that it will
    /// convert to a handle.
    ///
    /// The main benefit (besides convenience) of this over the version
    /// that takes a [`Handle`] is that it allows handling handle-related errors
    /// in one place.
    pub fn delete_u64(&self, h: u64) -> Result<(), HandleError> {
        self.delete(Handle::from_u64(h)?)
    }

    /// Remove an item from the map, returning either the item,
    /// or None if its guard mutex got poisoned at some point.
    ///
    /// This takes the map's `write` lock, and unwraps the `Arc`.
    pub fn remove(&self, h: Handle) -> Result<Option<T>, HandleError> {
        let mut map = self.map.write().unwrap();
        let arc = map.remove(h)?;
        match Arc::try_unwrap(arc) {
            Ok(obj) => Ok(Some(obj)),
            _ => Ok(None),
        }
    }

    /// Convenient wrapper for `remove` which takes a `u64` that it will
    /// convert to a handle.
    ///
    /// The main benefit (besides convenience) of this over the version
    /// that takes a [`Handle`] is that it allows handling handle-related errors
    /// in one place.
    pub fn remove_u64(&self, h: u64) -> Result<Option<T>, HandleError> {
        self.remove(Handle::from_u64(h)?)
    }

    /// Call `callback` with a non-mutable reference to the item from the map,
    /// after acquiring the necessary locks.
    ///
    /// This takes the map's `read` lock for as long as needed to clone the inner `Arc`.
    /// This is so the lock isn't held while the callback is in use.
    ///
    /// This takes the map's `read` lock for as long as needed to clone the inner `Arc`.
    /// This is so the lock isn't held while the callback is in use.
    pub fn get<F, E, R>(&self, h: Handle, callback: F) -> Result<R, E>
    where
        F: FnOnce(&T) -> Result<R, E>,
        E: From<HandleError>,
    {
        let obj = {
            let map = self.map.read().unwrap();
            let obj = map.get(h)?;
            Arc::clone(&obj)
        };
        callback(&*obj)
    }

    /// Convenient wrapper for `get` which takes a `u64` that it will convert to
    /// a handle.
    ///
    /// The other benefit (besides convenience) of this over the version
    /// that takes a [`Handle`] is that it allows handling handle-related errors
    /// in one place.
    ///
    /// This takes the map's `read` lock for as long as needed to clone the inner `Arc`.
    /// This is so the lock isn't held while the callback is in use.
    pub fn get_u64<F, E, R>(&self, u: u64, callback: F) -> Result<R, E>
    where
        F: FnOnce(&T) -> Result<R, E>,
        E: From<HandleError>,
    {
        self.get(Handle::from_u64(u)?, callback)
    }

    /// Helper that performs both a [`call_with_result`] and [`get`](ArcHandleMap::get).
    ///
    /// This takes the map's `read` lock for as long as needed to clone the inner `Arc`.
    /// This is so the lock isn't held while the callback is in use.
    pub fn call_with_result<R, E, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&T) -> Result<R, E>,
        ExternError: From<E>,
        R: IntoFfi,
    {
        use ffi_support::call_with_result;
        call_with_result(out_error, || -> Result<_, ExternError> {
            // We can't reuse `get` here because it would require E:
            // From<HandleError>, which is inconvenient...
            let h = Handle::from_u64(h)?;
            let obj = {
                let map = self.map.read().unwrap();
                let obj = map.get(h)?;
                Arc::clone(&obj)
            };
            Ok(callback(&*obj)?)
        })
    }

    /// Helper that performs both a [`call_with_output`] and [`get`](ArcHandleMap::get).
    pub fn call_with_output<R, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&T) -> R,
        R: IntoFfi,
    {
        self.call_with_result(out_error, h, |r| -> Result<_, HandleError> {
            Ok(callback(r))
        })
    }

    /// Use `constructor` to create and insert a `T`, while inside a
    /// [`call_with_result`] call (to handle panics and map errors onto an
    /// `ExternError`).
    ///
    /// This takes the map's `write` lock for as long as needed to insert into the map.
    /// This is so the lock isn't held while the constructor is being called.
    pub fn insert_with_result<E, F>(&self, out_error: &mut ExternError, constructor: F) -> u64
    where
        F: std::panic::UnwindSafe + FnOnce() -> Result<T, E>,
        ExternError: From<E>,
    {
        use ffi_support::call_with_result;
        call_with_result(out_error, || -> Result<_, ExternError> {
            // Note: it's important that we don't call the constructor while
            // we're holding the write lock, because we don't want to poison
            // the entire map if it panics!
            let to_insert = constructor()?;
            Ok(self.insert(to_insert))
        })
    }

    /// Equivalent to
    /// [`insert_with_result`](ArcHandleMap::insert_with_result) for the
    /// case where the constructor cannot produce an error.
    ///
    /// The name is somewhat dubious, since there's no `output`, but it's intended to make it
    /// clear that it contains a [`call_with_output`] internally.
    pub fn insert_with_output<F>(&self, out_error: &mut ExternError, constructor: F) -> u64
    where
        F: std::panic::UnwindSafe + FnOnce() -> T,
    {
        // The Err type isn't important here beyond being convertable to ExternError
        self.insert_with_result(out_error, || -> Result<_, HandleError> {
            Ok(constructor())
        })
    }
}

impl<T: Sync + Send> Default for ArcHandleMap<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Make a type alias to contrast the two handle map implementations.
pub type MutexHandleMap<T> = ffi_support::ConcurrentHandleMap<T>;

/// A trait to help the code generation a bit tidier.
///
/// We'll implement it only for the `MutexHandleMap` which asks for `FnOnce(&mut T)`
/// callbacks, but we'll give `ArcHandleMap` very similar looking methods, that
/// accept `FnOnce(&T)` callbacks.
///
/// When generating the code in the `to_rs_method_call` macro in `macros.rs`, the code will be lexically
/// identical.
pub trait UniffiMethodCall<T> {
    fn method_call_with_result<R, E, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&mut T) -> Result<R, E>,
        ExternError: From<E>,
        R: IntoFfi;

    fn method_call_with_output<R, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&mut T) -> R,
        R: IntoFfi;
}

impl<T> UniffiMethodCall<T> for MutexHandleMap<T> {
    fn method_call_with_result<R, E, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&mut T) -> Result<R, E>,
        ExternError: From<E>,
        R: IntoFfi,
    {
        self.call_with_result_mut(out_error, h, callback)
    }

    fn method_call_with_output<R, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&mut T) -> R,
        R: IntoFfi,
    {
        self.call_with_output_mut(out_error, h, callback)
    }
}

/// The faux implementation of `UniffiMethodCall` which differs from the real one
/// by not requiring `mut` references to `T`.
impl<T: Sync + Send> ArcHandleMap<T> {
    pub fn method_call_with_result<R, E, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&T) -> Result<R, E>,
        ExternError: From<E>,
        R: IntoFfi,
    {
        self.call_with_result(out_error, h, callback)
    }

    pub fn method_call_with_output<R, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&T) -> R,
        R: IntoFfi,
    {
        self.call_with_output(out_error, h, callback)
    }
}

/// Tests that check our behavior when panicking.
///
/// Naturally these require panic=unwind, which means we can't run them when
/// generating coverage (well, `-Zprofile`-based coverage can't -- although
/// ptrace-based coverage like tarpaulin can), and so we turn them off.
///
/// (For clarity, `cfg(coverage)` is not a standard thing. We add it in
/// `automation/emit_coverage_info.sh`, and you can force it by adding
/// "--cfg coverage" to your RUSTFLAGS manually if you need to do so).
///
/// Note: these tests are derived directly from ffi_support::ConcurrentHandleMap.
#[cfg(not(coverage))]
#[allow(unused_imports)]
mod panic_tests {
    use super::ArcHandleMap;
    use ffi_support::{call_with_result, ErrorCode, ExternError};

    #[derive(PartialEq, Debug)]
    pub(super) struct Foobar(usize);

    struct PanicOnDrop(());
    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!("intentional panic (drop)");
        }
    }

    #[test]
    fn test_panicking_drop() {
        let map = ArcHandleMap::new();
        let h = map.insert(PanicOnDrop(())).into_u64();
        let mut e = ExternError::success();
        call_with_result(&mut e, || map.delete_u64(h));
        assert_eq!(e.get_code(), ErrorCode::PANIC);
        let _ = unsafe { e.get_and_consume_message() };
        assert!(!map.map.is_poisoned());
        let inner = map.map.read().unwrap();
        assert_eq!(inner.len(), 0);
    }

    #[test]
    fn test_panicking_call_with() {
        let map = ArcHandleMap::new();
        let h = map.insert(Foobar(0)).into_u64();
        let mut e = ExternError::success();
        map.call_with_output(&mut e, h, |_thing| {
            panic!("intentional panic (call_with_output)");
        });

        assert_eq!(e.get_code(), ErrorCode::PANIC);
        let _ = unsafe { e.get_and_consume_message() };
        {
            assert!(!map.map.is_poisoned());
            let inner = map.map.read().unwrap();
            assert_eq!(inner.len(), 1);
        }
        assert!(map.delete_u64(h).is_ok());
        assert!(!map.map.is_poisoned());
        let inner = map.map.read().unwrap();
        assert_eq!(inner.len(), 0);
    }

    #[test]
    fn test_panicking_insert_with() {
        let map = ArcHandleMap::new();
        let mut e = ExternError::success();
        let res = map.insert_with_output(&mut e, || {
            panic!("intentional panic (insert_with_output)");
        });

        assert_eq!(e.get_code(), ErrorCode::PANIC);
        let _ = unsafe { e.get_and_consume_message() };

        assert_eq!(res, 0);

        assert!(!map.map.is_poisoned());
        let inner = map.map.read().unwrap();
        assert_eq!(inner.len(), 0);
    }
}
