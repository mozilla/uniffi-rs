/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Store Arc references owned by the foreign side and use handles to manage them
//!
//! This module defines the [Slab] class allows us to insert `Arc<>` values and use [Handle] values to manage the allocation.
//! It's named "Slab" because the data structure resembles a slab-allocator, for example the `tokio` `slab` crate (https://github.com/tokio-rs/slab).
//!
//! Usage:
//! * Create a `Slab` that will store Arc<T> values.
//! * Call `insert()` to store a value and allocated a handle that represents a single strong ref.
//! * Pass the handle across the FFI to the foreign side.
//! * When the foreign side wants to use that value, it passes back the handle back to Rust.
//! * If the FFI call treats the handle arg as a borrow, then Rust calls `get_clone` to get the stored value
//! * If the FFI call treats the handle arg as an owned value, then Rust calls `remove` to get the stored value and decrement the ref count.
//! * The foreign side can call `inc_ref` if they want to pass an owned reference back and continue to use the handle (See #1797)
//!
//! Using handles to manage arc references provides several benefits:
//! * Handles are simple integer values, which are simpler to work with than pointers.
//! * Handles store a generation counter, which can usually detect use-after-free bugs.
//! * Handles store an slab id, which can usually detect using handles with the wrong Slab.
//! * Handles only use 48 bits, which makes them easier to work with on languages like JS that don't support full 64-bit integers.
//! * Handles are signed, but always positive.  This allows using negative numbers for special values.
//!   Also, signed ints integrate with JNA easier.
//! * Handles have a bit to differentiate between foreign-allocated handles and rust-allocated ones.
//!   The trait interface code uses this to differentiate between Rust-implemented and foreign-implemented traits.

use std::fmt;

use append_only_vec::AppendOnlyVec;
use sync::*;

#[cfg(not(loom))]
mod sync {
    pub(super) use std::{
        sync::{
            atomic::{AtomicU8, Ordering},
            Mutex,
        },
        thread,
    };

    // Wrap UnsafeCell so that it has the same API as loom
    #[derive(Debug)]
    pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

    impl<T> UnsafeCell<T> {
        pub(crate) const fn new(data: T) -> UnsafeCell<T> {
            UnsafeCell(std::cell::UnsafeCell::new(data))
        }

        pub(crate) unsafe fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
            f(self.0.get())
        }

        pub(crate) unsafe fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
            f(self.0.get())
        }
    }
}

// Note: use the `cargo slab-loom-test` command to test with loom
#[cfg(loom)]
mod sync {
    pub(super) use loom::{
        cell::UnsafeCell,
        sync::{
            atomic::{AtomicU8, Ordering},
            Mutex,
        },
        thread,
    };
}

// This code assumes that usize is at least 32 bits
static_assertions::const_assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u32>());
// Entry should add 64 bits of storage for unit values, concrete `Arc<T>`, and `Arc<dyn Trait>`.
#[cfg(not(loom))]
static_assertions::const_assert!(std::mem::size_of::<Entry<()>>() == std::mem::size_of::<()>() + 8);
#[cfg(not(loom))]
static_assertions::const_assert!(
    std::mem::size_of::<Entry<std::sync::Arc<()>>>()
        == std::mem::size_of::<std::sync::Arc<()>>() + 8
);
#[cfg(not(loom))]
static_assertions::const_assert!(
    std::mem::size_of::<Entry<std::sync::Arc<dyn std::any::Any>>>()
        == std::mem::size_of::<std::sync::Arc<dyn std::any::Any>>() + 8
);

/// Slab error type
#[derive(Debug, PartialEq, Eq)]
pub enum SlabError {
    SlabIdMismatch,
    RustHandle,
    ForeignHandle,
    UseAfterFree(&'static str),
    OverCapacity,
    RefCountLimit,
    ReaderCountLimit,
    Vacant,
    OutOfBounds,
    LockTimeout,
}

impl fmt::Display for SlabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UseAfterFree(msg) => write!(
                f,
                "Slab error: {msg} (was the handle re-used after being passed to remove()?)"
            ),
            Self::SlabIdMismatch => write!(f, "Slab id mismatch"),
            Self::RustHandle => write!(f, "Handle belongs to a rust slab"),
            Self::ForeignHandle => write!(f, "Handle belongs to a foreign slab"),
            Self::OverCapacity => write!(f, "Slab capacity exceeded"),
            Self::RefCountLimit => write!(f, "Reference count limit exceeded"),
            Self::ReaderCountLimit => write!(f, "Reader count limit exceeded"),
            Self::Vacant => write!(f, "Entry unexpectedly vacant"),
            Self::OutOfBounds => write!(f, "Index out of bounds"),
            Self::LockTimeout => write!(f, "Lock timeout"),
        }
    }
}

impl std::error::Error for SlabError {}

pub type Result<T> = std::result::Result<T, SlabError>;

/// Index segment of a handle
const INDEX_MASK: i64 = 0x0000_FFFF_FFFF;
/// Foreign bit of a handle
const FOREIGN_BIT: i64 = 0x0001_0000_0000;
/// Special-cased value for the `next` field that means no next entry.
const END_OF_LIST: u32 = u32::MAX;

/// Handle for a value stored in the slab
///
/// * The first 32 bits identify the value.
/// * The next 8 bits are for an slab id:
///   - The first bit is 0 if the handle came from Rust and 1 if it came from the foreign side.
///   - The next 7 bits are used to identify the slab.  Use random values or a counter.
///   - This means that using a handle with the wrong Slab will be detected > 99% of the time.
/// * The next 8 bits are a generation counter value, this means that use-after-free bugs will be
///   detected until at least 256 inserts are performed after the free.
/// * The last 16 bits are intentionally unset, so that these can be easily used on languages like
///   JS that don't support full 64-bit integers.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(i64);

impl Handle {
    const fn new(slab_id: u8, generation: u8, index: u32) -> Self {
        Self((generation as i64) << 40 | (slab_id as i64) << 32 | index as i64)
    }

    pub const fn from_raw(val: i64) -> Self {
        Self(val)
    }

    pub const fn as_raw(&self) -> i64 {
        self.0
    }

    fn index(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }

    fn generation(&self) -> u8 {
        (self.0 >> 40) as u8
    }

    fn slab_id(&self) -> u8 {
        (self.0 >> 32) as u8
    }

    pub fn is_from_rust(&self) -> bool {
        self.0 & FOREIGN_BIT == 0
    }

    pub fn is_foreign(&self) -> bool {
        self.0 & FOREIGN_BIT != 0
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "handle-{}#{}", self.index(), self.generation())
    }
}

/// Entry a Slab
///
/// Entries can be vacant or occupied.
/// Vacant entries are part of the Slab's free list and don't have handles allocated for them.
/// Occupied entries are not part of the free list and are managed by handles.
///
/// Entries store a generation counter that's incremented each time it transitions from occupied to vacant.
/// Handles store the generation counter of the entry they were allocated for.
/// When handles are used, we check that the generation counters match.
/// This mostly ensures that use-after-free bugs are detected, although it's possible for the 8-bit counter to roll over.
#[derive(Debug)]
struct Entry<T: Clone> {
    /// For vacant entries, next entry in the free list.
    ///
    /// # Safety
    ///
    /// Only access this while [Slab::next_lock] is held.
    next: UnsafeCell<u32>,
    /// Generation counter
    generation: AtomicU8,
    /// Protects `ref_count` and `value`.
    ///
    /// Bit 0 is a write lock.
    /// Bits 1..8 form a reader count.
    /// The lock will only be contended if the foreign code uses a handle after it's been freed.
    state: AtomicU8,
    /// Reference count, this can be atomically updated by the readers after they've read-locked
    /// `state` and checked the generation value.  This is pretty small, but that's okay because
    /// it's only used to temporarily retain a reference that's being returned across the FFI (see
    /// #1797).
    ref_count: UnsafeCell<AtomicU8>,
    value: UnsafeCell<Option<T>>,
}

impl<T: Clone> Entry<T> {
    const WRITE_LOCK_BIT: u8 = 0x01;
    const READER_COUNT_UNIT: u8 = 0x02;
    // If ref_count or reader count get close to overflowing, then we should error out.
    //
    // Both of these numbers should never bit hit in practice.
    // Overflowing the ref count requires 200 threads to be suspended right after they returned the same handle, but before the Rust removed it.
    // Overflowing the reader count requires require 64 threads to be suspended in the middle of a `read()` operation, which are typically just a handful of CPU cycles.
    const REF_COUNT_LIMIT: u8 = 200;
    const READER_COUNT_LIMIT: u8 = Self::READER_COUNT_UNIT * 64;

    fn new_occupied(value: T) -> Self {
        Self {
            next: UnsafeCell::new(END_OF_LIST),
            state: AtomicU8::new(0),
            generation: AtomicU8::new(0),
            ref_count: UnsafeCell::new(AtomicU8::new(1)),
            value: UnsafeCell::new(Some(value)),
        }
    }

    fn acquire_read_lock(&self, handle: Handle) -> Result<()> {
        // Increment the reader count. Use a spin lock to wait for writers.  As long as the foreign
        // code isn't using handles after they're freed, there will never be contention.
        let mut counter = 0;
        loop {
            let prev_state = self
                .state
                .fetch_add(Self::READER_COUNT_UNIT, Ordering::Acquire);
            if !self.generation_matches(handle) {
                self.release_read_lock();
                return Err(SlabError::UseAfterFree("generation mismatch"));
            } else if prev_state >= Self::READER_COUNT_LIMIT {
                self.release_read_lock();
                return Err(SlabError::ReaderCountLimit);
            } else if prev_state & Self::WRITE_LOCK_BIT == 0 {
                return Ok(());
            }
            self.release_read_lock();
            // As mentioned above, the lock should never be contended and locks are only held for a
            // handful of instructions, so let's use an extremely simple solution to manage
            // contention.
            if counter < 100 {
                thread::yield_now();
                counter += 1;
            } else {
                return Err(SlabError::LockTimeout);
            }
        }
    }

    fn release_read_lock(&self) {
        self.state
            .fetch_sub(Self::READER_COUNT_UNIT, Ordering::Release);
    }

    fn acquire_write_lock(&self) -> Result<()> {
        // Set the write lock bit. Use a spin lock to wait for writers and readers. As long as the
        // foreign code isn't using handles after they're freed, there will never be contention.
        let mut counter = 0;
        while self
            .state
            .compare_exchange_weak(
                0,
                Self::WRITE_LOCK_BIT,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            // See `acquire_read_lock` for notes on this.
            if counter < 100 {
                thread::yield_now();
                counter += 1;
            } else {
                return Err(SlabError::LockTimeout);
            }
        }
        Ok(())
    }

    fn release_write_lock(&self) {
        self.state
            .fetch_and(!Self::WRITE_LOCK_BIT, Ordering::Release);
    }

    fn generation_matches(&self, handle: Handle) -> bool {
        self.generation.load(Ordering::Relaxed) == handle.generation()
    }

    /// Perform a operation with the read lock
    fn read<F, R>(&self, handle: Handle, f: F) -> Result<R>
    where
        F: FnOnce(&AtomicU8, &Option<T>) -> Result<R>,
    {
        self.acquire_read_lock(handle)?;
        let result = unsafe {
            // Safety: We hold a read lock
            self.ref_count
                .with(|ref_count| self.value.with(|v| f(&*ref_count, &*v)))
        };
        self.release_read_lock();
        result
    }

    /// Perform an operation with the write lock
    ///
    /// This is marked unsafe because it does not check the generation.   Only call this if you
    /// know that you should have access to the entry.
    unsafe fn write<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AtomicU8, &mut Option<T>),
    {
        self.acquire_write_lock()?;
        unsafe {
            // Safety: We hold the write lock
            self.ref_count
                .with_mut(|ref_count| self.value.with_mut(|v| f(&mut *ref_count, &mut *v)))
        };
        self.release_write_lock();
        Ok(())
    }

    /// Increment the ref count
    fn inc_ref(&self, handle: Handle) -> Result<()> {
        // Increment the ref count inside `read` to ensure the generation counter matches
        self.read(handle, |ref_count, _| {
            let prev_ref_count = ref_count.fetch_add(1, Ordering::Relaxed);
            if prev_ref_count >= Self::REF_COUNT_LIMIT {
                ref_count.fetch_sub(1, Ordering::Relaxed);
                Err(SlabError::RefCountLimit)
            } else {
                Ok(())
            }
        })
    }

    /// Get a cloned value
    fn get_clone(&self, handle: Handle) -> Result<T> {
        // Decrement the ref count inside `read` to ensure the generation counter matches.
        self.read(handle, |_, value| match value {
            Some(v) => Ok(v.clone()),
            None => Err(SlabError::Vacant),
        })
    }

    /// Remove a reference
    ///
    /// Returns the inner value plus an extra `needs_free` flag which indicates that
    /// the entry should be return to the free list.
    fn remove(&self, handle: Handle) -> Result<(T, bool)> {
        // Decrement the ref count inside `read` to ensure the generation counter matches.
        self.read(handle, |ref_count, value| {
            let value = match value {
                Some(v) => v.clone(),
                None => return Err(SlabError::Vacant),
            };
            let needs_free = ref_count.fetch_sub(1, Ordering::Relaxed) == 1;
            Ok((value, needs_free))
        })
        .and_then(|(v, needs_free)| {
            if needs_free {
                // make_vacant() should never fail here as long as our internal logic is correct.
                self.make_vacant(handle)?;
            }
            Ok((v, needs_free))
        })
    }

    /// Transition an entry to vacant
    fn make_vacant(&self, handle: Handle) -> Result<()> {
        self.generation
            .compare_exchange_weak(
                handle.generation(),
                handle.generation().wrapping_add(1),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .map_err(|_| SlabError::UseAfterFree("simultaneous frees"))?;

        // Safety: we successfully incremented the generation counter, so we know that our handle
        // was valid for the entry.
        unsafe {
            self.write(|_, value| {
                *value = None;
            })
        }
    }

    /// Transition an entry to occupied and return the generation value
    ///
    /// # Safety
    ///
    /// Must only be called on vacant entries that have been removed the free list before any new
    /// handles are allocated.
    unsafe fn make_occupied(&self, new_value: T) -> Result<u8> {
        // Safety: the entry was just removed from the free list, so we have access to it
        unsafe {
            self.write(|ref_count, value| {
                *value = Some(new_value);
                ref_count.store(1, Ordering::Relaxed);
            })?;
        }
        Ok(self.generation.load(Ordering::Relaxed))
    }
}

/// Allocates handles that represent stored values and can be shared by the foreign code
pub struct Slab<T: Clone> {
    is_foreign: bool,
    // Slab ID, including the foreign bit
    slab_id: u8,
    // Use an append-only vec, which has the nice property that we can push to it with a shared
    // reference
    entries: AppendOnlyVec<Entry<T>>,
    // Next entry in the free list.
    next: UnsafeCell<u32>,
    // Protects [Slab::next] and the [Entry::next] field for all entries in the slab.
    next_lock: Mutex<()>,
}

impl<T: Clone> Slab<T> {
    #[cfg(not(loom))]
    pub const fn new_with_id_and_foreign(slab_id: u8, is_foreign: bool) -> Self {
        Self {
            slab_id: if is_foreign {
                (slab_id << 1) | 1
            } else {
                slab_id << 1
            },
            is_foreign,
            entries: AppendOnlyVec::new(),
            next: UnsafeCell::new(END_OF_LIST),
            next_lock: Mutex::new(()),
        }
    }

    /// This needs to be non-const because loom's AtomicU32::new() is non-const.
    #[cfg(loom)]
    pub fn new_with_id_and_foreign(slab_id: u8, is_foreign: bool) -> Self {
        Self {
            slab_id: if is_foreign {
                (slab_id << 1) | 1
            } else {
                slab_id << 1
            },
            is_foreign,
            entries: AppendOnlyVec::new(),
            next: UnsafeCell::new(END_OF_LIST),
            next_lock: Mutex::new(()),
        }
    }

    /// Get an entry for a handle, if the handle is still valid
    fn get_entry(&self, handle: Handle) -> Result<&Entry<T>> {
        let index = handle.index();
        if handle.slab_id() != self.slab_id {
            if handle.is_foreign() && !self.is_foreign {
                return Err(SlabError::ForeignHandle);
            } else if !handle.is_foreign() && self.is_foreign {
                return Err(SlabError::RustHandle);
            } else {
                return Err(SlabError::SlabIdMismatch);
            }
        }
        if index < self.entries.len() {
            Ok(&self.entries[index])
        } else {
            Err(SlabError::OutOfBounds)
        }
    }

    /// Insert a new item into the Slab, either by pushing it to the end or re-allocating a previously removed entry.
    pub fn insert(&self, value: T) -> Result<Handle> {
        let _guard = self.next_lock.lock().unwrap();
        unsafe {
            // Safety: we hold `next_lock`
            self.next.with_mut(|next| {
                if *next == END_OF_LIST {
                    // No vacant entries, create a new one
                    if self.entries.len() + 1 >= END_OF_LIST as usize {
                        // ~4 billion entries allocated, a new one will overflow the bits available
                        // in the handle.
                        Err(SlabError::OverCapacity)
                    } else {
                        let index = self.entries.push(Entry::new_occupied(value));
                        Ok(Handle::new(self.slab_id, 0, index as u32))
                    }
                } else {
                    // Pop a vacant entry off the free list
                    let entry_index = *next;
                    let entry = &self.entries[entry_index as usize];
                    // Safety: we hold `next_lock`
                    entry.next.with(|entry_next| *next = *entry_next);
                    // Safety:
                    //
                    // We have removed entry from the free list and not allocated any
                    // handles yet.
                    //
                    // make_occupied() should never fail here as long as our internal logic is
                    // correct.
                    let generation = entry.make_occupied(value)?;
                    Ok(Handle::new(self.slab_id, generation, entry_index))
                }
            })
        }
    }

    /// Get a cloned value from a handle
    pub fn get_clone(&self, handle: Handle) -> Result<T> {
        self.get_entry(handle)?.get_clone(handle)
    }

    /// Increment the reference count
    pub fn inc_ref(&self, handle: Handle) -> Result<()> {
        self.get_entry(handle)?.inc_ref(handle)
    }

    /// Remove a reference
    ///
    /// This decrements the reference count, returns the inner value and if the entry was freed
    pub fn remove(&self, handle: Handle) -> Result<(T, bool)> {
        let entry = self.get_entry(handle)?;
        entry.remove(handle).and_then(|(v, needs_free)| {
            if needs_free {
                self.free_entry(handle, entry)?;
            }
            Ok((v, needs_free))
        })
    }

    /// Add an entry back to the free list
    fn free_entry(&self, handle: Handle, entry: &Entry<T>) -> Result<()> {
        let _guard = self.next_lock.lock().unwrap();
        unsafe {
            // Safety: we hold `next_lock'
            self.next.with_mut(|next| {
                // Safety: we hold `next_lock'
                entry.next.with_mut(|entry_next| {
                    *entry_next = *next;
                    *next = handle.index() as u32;
                })
            });
        }
        Ok(())
    }

    pub fn insert_or_panic(&self, value: T) -> Handle {
        self.insert(value).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn get_clone_or_panic(&self, handle: Handle) -> T {
        self.get_clone(handle).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn remove_or_panic(&self, handle: Handle) -> (T, bool) {
        self.remove(handle).unwrap_or_else(|e| panic!("{e}"))
    }
}

// If the code above is correct, then Slab is Send + Sync
unsafe impl<T: Clone> Send for Slab<T> {}
unsafe impl<T: Clone> Sync for Slab<T> {}

#[cfg(test)]
impl<T: Clone> Entry<T> {
    fn reader_count(&self) -> u8 {
        self.state.load(Ordering::Relaxed) / Self::READER_COUNT_UNIT
    }

    fn ref_count(&self) -> u8 {
        unsafe {
            self.ref_count
                .with(|ref_count| (*ref_count).load(Ordering::Relaxed))
        }
    }
}

#[cfg(test)]
mod entry_tests {
    use super::*;
    use std::sync::{Arc, Weak};

    fn test_setup() -> (Entry<Arc<()>>, Handle, Weak<()>) {
        let obj = Arc::new(());
        let weak = Arc::downgrade(&obj);
        let entry = Entry::new_occupied(obj);
        let handle = Handle::new(0, 0, 0);
        (entry, handle, weak)
    }

    #[test]
    fn test_ref_count() {
        let (entry, handle, weak) = test_setup();
        assert_eq!(entry.ref_count(), 1);
        entry.inc_ref(handle).unwrap();
        assert_eq!(entry.ref_count(), 2);
        let needs_free = entry.remove(handle).unwrap().1;
        assert_eq!(entry.ref_count(), 1);
        assert!(!needs_free);
        let needs_free = entry.remove(handle).unwrap().1;
        assert!(needs_free);
        assert_eq!(weak.strong_count(), 0);
    }

    #[test]
    fn test_extra_release() {
        let (entry, handle, _) = test_setup();
        entry.remove(handle).unwrap();
        assert!(entry.remove(handle).is_err());
    }

    // Test that incrementing the reader count fails before getting close to the limit
    #[test]
    fn test_ref_count_overflow() {
        // Create an entry with ref_count = 1
        let (entry, handle, weak) = test_setup();
        // Incrementing this many times is okay
        for _ in 0..199 {
            entry.inc_ref(handle).unwrap();
        }
        // 1 more should fail because it gets us too close the limit where we run out of bits
        assert_eq!(entry.inc_ref(handle), Err(SlabError::RefCountLimit));
        // If we remove the references then the value should be freed.
        for _ in 0..200 {
            entry.remove(handle).unwrap();
        }
        assert_eq!(weak.strong_count(), 0);
    }

    // Test that incrementing the reader count fails before getting close to the limits
    #[test]
    fn test_reader_overflow() {
        // 800 readers are okay
        let (entry, handle, _) = test_setup();
        for _ in 0..64 {
            entry.acquire_read_lock(handle).unwrap();
        }
        // 1 more should fail because it gets us too close to the limit where we run out of bits
        assert_eq!(entry.inc_ref(handle), Err(SlabError::ReaderCountLimit));
        // Test decrementing the reader count
        for _ in 0..64 {
            entry.release_read_lock();
        }
        assert_eq!(entry.reader_count(), 0);
    }
}

#[cfg(test)]
mod slab_tests {
    use super::*;
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use std::sync::Arc;

    #[test]
    fn test_simple_usage() {
        let slab = Slab::new_with_id_and_foreign(0, false);
        let handle1 = slab.insert(Arc::new("Hello")).unwrap();
        let handle2 = slab.insert(Arc::new("Goodbye")).unwrap();
        assert_eq!(slab.entries.len(), 2);
        assert_eq!(*slab.get_clone(handle1).unwrap(), "Hello");
        slab.remove(handle1).unwrap();
        assert_eq!(*slab.get_clone(handle2).unwrap(), "Goodbye");
        slab.remove(handle2).unwrap();
    }

    #[test]
    fn test_slab_id_check() {
        let slab = Slab::<Arc<&str>>::new_with_id_and_foreign(1, false);
        let slab2 = Slab::<Arc<&str>>::new_with_id_and_foreign(2, false);
        let handle = slab.insert(Arc::new("Hello")).unwrap();
        assert_eq!(Err(SlabError::SlabIdMismatch), slab2.get_clone(handle));
        assert_eq!(Err(SlabError::SlabIdMismatch), slab2.remove(handle));
    }

    #[test]
    fn test_foreign_handle_with_rust_slab() {
        let slab = Slab::<Arc<&str>>::new_with_id_and_foreign(1, false);
        let handle = slab.insert(Arc::new("Hello")).unwrap();
        let foreign_handle = Handle::from_raw(handle.as_raw() | FOREIGN_BIT);
        assert_eq!(
            Err(SlabError::ForeignHandle),
            slab.get_clone(foreign_handle)
        );
    }

    #[test]
    fn test_rust_handle_with_foreign_slab() {
        let slab = Slab::<Arc<&str>>::new_with_id_and_foreign(1, true);
        let handle = slab.insert(Arc::new("Hello")).unwrap();
        let rust_handle = Handle::from_raw(handle.as_raw() & !FOREIGN_BIT);
        assert_eq!(Err(SlabError::RustHandle), slab.get_clone(rust_handle));
    }

    fn rand_index<T>(rng: &mut StdRng, vec: &Vec<T>) -> usize {
        rng.next_u32() as usize % vec.len()
    }

    // Wraps a slab for easier testing
    #[derive(Clone)]
    pub struct TestSlab {
        slab: Arc<Slab<u8>>,
        counter: Arc<AtomicU8>,
    }

    impl TestSlab {
        pub fn new() -> Self {
            Self {
                slab: Arc::new(Slab::new_with_id_and_foreign(0, false)),
                counter: Arc::new(AtomicU8::new(0)),
            }
        }

        pub fn insert(&self) -> TestHandle {
            let value = self.counter.fetch_add(1, Ordering::Relaxed);
            let handle = self.slab.insert(value).unwrap();
            TestHandle {
                handle,
                value,
                ref_count: 1,
            }
        }

        pub fn check(&self, handle: &TestHandle) {
            let value = self.slab.get_clone(handle.handle).unwrap();
            assert_eq!(value, handle.value);
        }

        pub fn inc_ref(&self, handle: &mut TestHandle) {
            self.slab.inc_ref(handle.handle).unwrap();
            handle.ref_count += 1;
        }

        pub fn remove(&self, handle: &mut TestHandle) -> bool {
            handle.ref_count -= 1;
            let (value, freed) = self.slab.remove(handle.handle).unwrap();
            assert_eq!(value, handle.value);
            assert_eq!(freed, handle.ref_count == 0);
            freed
        }

        pub fn check_use_after_free_detection(&self, handle: &TestHandle) {
            let result = self.slab.get_clone(handle.handle);
            assert!(
                matches!(result, Err(SlabError::UseAfterFree(_))),
                "{result:?}"
            );
        }
    }

    // Store a handle, it's entry's value, and it's ref count together
    pub struct TestHandle {
        pub handle: Handle,
        pub value: u8,
        pub ref_count: u8,
    }

    impl fmt::Debug for TestHandle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.handle.fmt(f)
        }
    }

    #[test]
    fn stress_test() {
        let mut rng = StdRng::seed_from_u64(42);
        for i in 0..100 {
            println!("---------------------- {i} ------------------------");
            let slab = TestSlab::new();
            let mut allocated = vec![];
            let mut freed = vec![];
            // Note; the inner loop is 255 elements, because that's the limit of insertions before
            // our use-after-free detection can fail.
            for _ in 0..255 {
                // Insert or remove a handle
                let roll = rng.next_u32() % 3;
                if allocated.is_empty() || roll == 0 {
                    // Insert
                    println!("slab.insert()");
                    let handle = slab.insert();
                    println!("{handle:?}: handle");
                    allocated.push(handle);
                } else if roll == 2 {
                    // inc_ref
                    let idx = rand_index(&mut rng, &allocated);
                    let handle = &mut allocated[idx];
                    println!("{handle:?}: inc_ref");
                    slab.inc_ref(handle);
                } else {
                    // Remove
                    let idx = rand_index(&mut rng, &allocated);
                    let handle = &mut allocated[idx];
                    println!("{handle:?}: remove");
                    if slab.remove(handle) {
                        println!("{handle:?}: freed");
                        freed.push(allocated.remove(idx));
                    }
                }

                // Test getting all handles, allocated or freed
                for handle in allocated.iter() {
                    println!("{handle:?}: check");
                    slab.check(handle);
                }
                for handle in freed.iter() {
                    println!("{handle:?}: check_use_after_free_detection");
                    slab.check_use_after_free_detection(handle);
                }
            }
        }
    }
}

#[cfg(loom)]
mod slab_loom_test {
    use super::slab_tests::{TestHandle, TestSlab};
    use super::*;
    use loom::{
        model::Builder,
        sync::{atomic::AtomicU64, Arc},
        thread,
    };

    // Simple tracing for the loom tests.
    macro_rules! trace {
        ($($tt:tt)*) => {
            println!("{:?}: {}", thread::current().id(), format!($($tt)*));
        }
    }

    // In these tests we're going to swap handles using AtomicU64
    impl TestHandle {
        pub fn as_raw(&self) -> u64 {
            self.handle.as_raw() as u64 | (self.value as u64) << 48 | (self.ref_count as u64) << 56
        }

        pub fn from_raw(raw: u64) -> Self {
            Self {
                handle: Handle::from_raw((raw & 0xFFFF_FFFF_FFFF) as i64),
                value: ((raw >> 48) & 0xFF) as u8,
                ref_count: ((raw >> 56) & 0xFF) as u8,
            }
        }

        pub fn swap(&mut self, shared: &AtomicU64) {
            let raw = shared.swap(self.as_raw(), Ordering::AcqRel);
            *self = Self::from_raw(raw)
        }
    }

    /// Test a set of threads that shares handles between themselves
    ///
    /// This runs the same basic test with different parameters.  These numbers may seem low, but
    /// they cause loom to run a tens of thousands of combinations.
    #[test]
    fn test_shared_handles() {
        // Test with less threads but a higher preemption bound
        test_shared_handles_case(2, 4, 3);
        // Test with more threads, but a lower preemption bound
        test_shared_handles_case(3, 4, 2);
    }

    fn test_shared_handles_case(thread_count: usize, iterations: usize, preemption_bound: usize) {
        let mut builder = Builder::default();
        builder.max_branches = 10_000;
        // Limit the number of times a thread can be pre-empted.  This severely limits the number
        // of iterations loom needs to run.  The `loom` docs say "2-3 is enough to catch most
        // bugs", and this has been true in testing.  Let's stay slightly on the cautious side and
        // set it to 4.
        builder.preemption_bound = Some(preemption_bound);
        let iteration_counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        builder.check(move || {
            trace!(
                "---------------------- {} -----------------------------",
                iteration_counter.fetch_add(1, Ordering::Relaxed)
            );
            let slab = TestSlab::new();
            // Used to share handles between threads
            let shared = Arc::new(AtomicU64::new(slab.insert().as_raw()));
            for _ in 0..thread_count {
                let slab = slab.clone();
                let shared = shared.clone();
                thread::spawn(move || {
                    trace!("startup");
                    let mut current = slab.insert();
                    trace!("{current:?}: initial handle");
                    let mut freed_handles = vec![];
                    for _ in 0..iterations {
                        trace!("{current:?}: swapping out");
                        current.swap(&shared);
                        trace!("{current:?}: inc_ref");
                        slab.inc_ref(&mut current);
                        trace!("{current:?}: check");
                        slab.check(&current);
                        // Swap and dec-ref
                        trace!("{current:?}: swapping out");
                        current.swap(&shared);
                        trace!("{current:?}: remove");
                        let freed = slab.remove(&mut current);
                        trace!("{current:?}: {}", if freed { "freed" } else { "live" });
                        if freed {
                            freed_handles.push(current);
                            trace!("inserting new handle");
                            current = slab.insert();
                            trace!("{current:?}: new handle");
                        }
                        // Check all freed handles
                        for freed in &freed_handles {
                            trace!("{freed:?}: get_clone for freed handle check");
                            slab.check_use_after_free_detection(freed);
                        }
                        trace!("loop done");
                    }
                });
            }
        })
    }

    /// Test two threads calling `remove` when there's only 1 reference
    #[test]
    fn test_extra_remove() {
        loom::model(|| {
            let slab = Arc::new(Slab::new_with_id_and_foreign(0, false));
            let slab2 = Arc::clone(&slab);
            let handle = slab.insert(42).unwrap();

            let result1 = thread::spawn(move || slab.remove(handle)).join().unwrap();
            let result2 = thread::spawn(move || slab2.remove(handle)).join().unwrap();
            // One remove should succeed and one should fail with `SlabError::UseAfterFree`
            match (&result1, &result2) {
                (Ok((42, true)), Err(SlabError::UseAfterFree(_)))
                | (Err(SlabError::UseAfterFree(_)), Ok((42, true))) => (),
                _ => panic!("Unexpected results: ({result1:?}, {result2:?})"),
            }
        })
    }

    /// Test one threads calling `remove`` and one calling `get_clone` when there's only 1 reference
    #[test]
    fn test_get_with_extra_remove() {
        loom::model(|| {
            let slab = Arc::new(Slab::new_with_id_and_foreign(0, false));
            let slab2 = Arc::clone(&slab);
            let handle = slab.insert(42).unwrap();

            let result1 = thread::spawn(move || slab.get_clone(handle))
                .join()
                .unwrap();
            let result2 = thread::spawn(move || slab2.remove(handle)).join().unwrap();
            // `get_clone` may or may not succeed, remove should always succeed
            match (&result1, &result2) {
                (Ok(42), Ok((42, true))) | (Err(SlabError::UseAfterFree(_)), Ok((42, true))) => (),
                _ => panic!("Unexpected results: ({result1:?}, {result2:?})"),
            }
        })
    }

    /// Test various combinations of:
    ///   * an extra `remove`,
    ///   * a `get_clone`
    ///   * An `insert` that may re-allocate the entry
    #[test]
    fn test_invalid_access_combos() {
        loom::model(|| {
            let slab = Arc::new(Slab::new_with_id_and_foreign(0, false));
            let slab2 = Arc::clone(&slab);
            let slab3 = Arc::clone(&slab);
            let slab4 = Arc::clone(&slab);
            let handle = slab.insert(42).unwrap();

            let result1 = thread::spawn(move || slab.get_clone(handle))
                .join()
                .unwrap();
            let result2 = thread::spawn(move || slab2.remove(handle)).join().unwrap();
            let result3 = thread::spawn(move || slab3.remove(handle)).join().unwrap();
            let result4 = thread::spawn(move || slab4.insert(43)).join().unwrap();
            // * `get_clone` may or may not succeed
            // * One of the `remove` calls should succeed
            // * `insert` should always succeed
            match &result1 {
                Ok(42) | Err(SlabError::UseAfterFree(_)) => (),
                _ => panic!("Unexpected get_clone() result: {result1:?}"),
            }
            match (&result2, &result3) {
                (Ok((42, true)), Err(SlabError::UseAfterFree(_)))
                | (Err(SlabError::UseAfterFree(_)), Ok((42, true))) => (),
                _ => panic!("Unexpected remove() results: ({result2:?}, {result3:?})"),
            }
            match &result4 {
                Ok(_) => (),
                _ => panic!("Unexpected insert() result: {result4:?}"),
            }
        })
    }
}
