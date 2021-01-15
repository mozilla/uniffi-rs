use ffi_support::ExternError;
use ffi_support::Handle;
use ffi_support::HandleError;
use ffi_support::HandleMap;
use ffi_support::IntoFfi;
use std::sync::Mutex;
use std::sync::RwLock;

pub struct NonLockingHandleMap<T> {
    /// The underlying map. Public so that more advanced use-cases
    /// may use it as they please.
    pub map: RwLock<HandleMap<Mutex<T>>>,
}

impl<T> NonLockingHandleMap<T> {
    /// Construct a new `NonLockingHandleMap`.
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HandleMap::new()),
        }
    }

    /// Get the number of entries in the `NonLockingHandleMap`.
    ///
    /// This takes the map's `read` lock.
    #[inline]
    pub fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }

    /// Returns true if the `NonLockingHandleMap` is empty.
    ///
    /// This takes the map's `read` lock.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert an item into the map, returning the newly allocated handle to the
    /// item.
    ///
    /// # Locking
    ///
    /// Note that this requires taking the map's write lock, and so it will
    /// block until all other threads have finished any read/write operations.
    pub fn insert(&self, v: T) -> Handle {
        // Fails if the lock is poisoned. Not clear what we should do here... We
        // could always insert anyway (by matching on LockResult), but that
        // seems... really quite dubious.
        let mut map = self.map.write().unwrap();
        map.insert(Mutex::new(v))
    }

    /// Remove an item from the map.
    ///
    /// # Locking
    ///
    /// Note that this requires taking the map's write lock, and so it will
    /// block until all other threads have finished any read/write operations.
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
    /// # Locking
    ///
    /// Note that this requires taking the map's write lock, and so it will
    /// block until all other threads have finished any read/write operations.
    pub fn remove(&self, h: Handle) -> Result<Option<T>, HandleError> {
        let mut map = self.map.write().unwrap();
        let mutex = map.remove(h)?;
        Ok(mutex.into_inner().ok())
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
    /// # Locking
    ///
    /// Note that this requires taking both:
    ///
    /// - The map's read lock, and so it will block until all other threads have
    ///   finished any write operations.
    /// - The mutex on the slot the handle is mapped to.
    ///
    /// And so it will block if there are ongoing write operations, or if
    /// another thread is reading from the same handle.
    ///
    /// # Panics
    ///
    /// This will panic if a previous `get()` or `get_mut()` call has panicked
    /// inside it's callback. The solution to this
    ///
    /// (It may also panic if the handle map detects internal state corruption,
    /// however this should not happen except for bugs in the handle map code).
    pub fn get<F, E, R>(&self, h: Handle, callback: F) -> Result<R, E>
    where
        F: FnOnce(&T) -> Result<R, E>,
        E: From<HandleError>,
    {
        self.get_mut(h, |v| callback(v))
    }

    /// Call `callback` with a mutable reference to the item from the map, after
    /// acquiring the necessary locks.
    ///
    /// # Locking
    ///
    /// Note that this requires taking both:
    ///
    /// - The map's read lock, and so it will block until all other threads have
    ///   finished any write operations.
    /// - The mutex on the slot the handle is mapped to.
    ///
    /// And so it will block if there are ongoing write operations, or if
    /// another thread is reading from the same handle.
    ///
    /// # Panics
    ///
    /// This will panic if a previous `get()` or `get_mut()` call has panicked
    /// inside it's callback. The only solution to this is to remove and reinsert
    /// said item.
    ///
    /// (It may also panic if the handle map detects internal state corruption,
    /// however this should not happen except for bugs in the handle map code).
    pub fn get_mut<F, E, R>(&self, h: Handle, callback: F) -> Result<R, E>
    where
        F: FnOnce(&mut T) -> Result<R, E>,
        E: From<HandleError>,
    {
        // XXX figure out how to handle poison...
        let map = self.map.read().unwrap();
        let mtx = map.get(h)?;
        let mut hm = mtx.lock().unwrap();
        callback(&mut *hm)
    }

    /// Convenient wrapper for `get` which takes a `u64` that it will convert to
    /// a handle.
    ///
    /// The other benefit (besides convenience) of this over the version
    /// that takes a [`Handle`] is that it allows handling handle-related errors
    /// in one place.
    ///
    /// # Locking
    ///
    /// Note that this requires taking both:
    ///
    /// - The map's read lock, and so it will block until all other threads have
    ///   finished any write operations.
    /// - The mutex on the slot the handle is mapped to.
    ///
    /// And so it will block if there are ongoing write operations, or if
    /// another thread is reading from the same handle.
    pub fn get_u64<F, E, R>(&self, u: u64, callback: F) -> Result<R, E>
    where
        F: FnOnce(&T) -> Result<R, E>,
        E: From<HandleError>,
    {
        self.get(Handle::from_u64(u)?, callback)
    }

    /// Convenient wrapper for `get_mut` which takes a `u64` that it will
    /// convert to a handle.
    ///
    /// The main benefit (besides convenience) of this over the version
    /// that takes a [`Handle`] is that it allows handling handle-related errors
    /// in one place.
    ///
    /// # Locking
    ///
    /// Note that this requires taking both:
    ///
    /// - The map's read lock, and so it will block until all other threads have
    ///   finished any write operations.
    /// - The mutex on the slot the handle is mapped to.
    ///
    /// And so it will block if there are ongoing write operations, or if
    /// another thread is reading from the same handle.
    pub fn get_mut_u64<F, E, R>(&self, u: u64, callback: F) -> Result<R, E>
    where
        F: FnOnce(&mut T) -> Result<R, E>,
        E: From<HandleError>,
    {
        self.get_mut(Handle::from_u64(u)?, callback)
    }

    /// Helper that performs both a [`call_with_result`] and [`get`](NonLockingHandleMap::get_mut).
    pub fn call_with_result_mut<R, E, F>(
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
        use ffi_support::call_with_result;
        call_with_result(out_error, || -> Result<_, ExternError> {
            // We can't reuse get_mut here because it would require E:
            // From<HandleError>, which is inconvenient...
            let h = Handle::from_u64(h)?;
            let map = self.map.read().unwrap();
            let mtx = map.get(h)?;
            let mut hm = mtx.lock().unwrap();
            Ok(callback(&mut *hm)?)
        })
    }

    /// Helper that performs both a [`call_with_result`] and [`get`](NonLockingHandleMap::get).
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
        self.call_with_result_mut(out_error, h, |r| callback(r))
    }

    /// Helper that performs both a [`call_with_output`] and [`get`](NonLockingHandleMap::get).
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

    /// Helper that performs both a [`call_with_output`] and [`get_mut`](NonLockingHandleMap::get).
    pub fn call_with_output_mut<R, F>(
        &self,
        out_error: &mut ExternError,
        h: u64,
        callback: F,
    ) -> R::Value
    where
        F: std::panic::UnwindSafe + FnOnce(&mut T) -> R,
        R: IntoFfi,
    {
        self.call_with_result_mut(out_error, h, |r| -> Result<_, HandleError> {
            Ok(callback(r))
        })
    }

    /// Use `constructor` to create and insert a `T`, while inside a
    /// [`call_with_result`] call (to handle panics and map errors onto an
    /// `ExternError`).
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
    /// [`insert_with_result`](NonLockingHandleMap::insert_with_result) for the
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

impl<T> Default for NonLockingHandleMap<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}