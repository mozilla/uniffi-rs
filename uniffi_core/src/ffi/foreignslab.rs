/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Slab functionality for the foreign bindings
//!
//! This module creates Slab instances that the foreign bindings can use.
//! This slabs don't store anything, they just manage the handles.
//! The foreign code creates the actual object array and stores objects using the handle's index.
//! For most languages, the array is protected with a read-write lock.
//! Overall, this is a slightly less efficient system, but it's simple for bindings to use.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{derive_ffi_traits, Handle, Slab, SlabAlloc};

type ForeignSlab = Slab<()>;

static SLAB_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

// How should we allocate and manage [ForeignSlab] instances?  Another Slab.
derive_ffi_traits!(impl<UT> SlabAlloc<UT> for ForeignSlab);

// ==================== Public API ====================
//
// All of these functions are exported as a extern "C" function in the scaffolding code

/// Create a new slab and get a handle for the slab
pub fn uniffi_slab_new() -> Handle {
    let slab_id = (SLAB_ID_COUNTER.fetch_add(1, Ordering::Relaxed) & 0xFF) as u8;
    <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::insert(Arc::new(
        ForeignSlab::new_with_id_and_foreign(slab_id, true),
    ))
}

/// Free a slab
pub fn uniffi_slab_free(slab: Handle) {
    <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::remove(slab);
}

/// Insert a new entry and get a handle for it
///
/// Returns the handle on success, -1 on error
pub fn uniffi_slab_insert(slab: Handle) -> i64 {
    let slab = <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::get_clone(slab);
    match slab.insert(()) {
        Ok(handle) => handle.as_raw(),
        Err(e) => {
            println!("{e}");
            -1
        }
    }
}

/// Check that a handle is still valid
///
/// Returns 0 on success, -1 on error
pub fn uniffi_slab_check_handle(slab: Handle, handle: Handle) -> i8 {
    let slab = <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::get_clone(slab);
    match slab.get_clone(handle) {
        Ok(_) => 0,
        Err(e) => {
            println!("{e}");
            -1
        }
    }
}

/// Increment the handle reference count
///
/// Returns 0 on success, -1 on error
pub fn uniffi_slab_inc_ref(slab: Handle, handle: Handle) -> i8 {
    let slab = <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::get_clone(slab);
    match slab.inc_ref(handle) {
        Ok(_) => 0,
        Err(e) => {
            println!("{e}");
            -1
        }
    }
}

/// Decrement the handle reference count
///
/// Returns 1 if the handle should be freed, 0 if not, and -1 on error.
pub fn uniffi_slab_dec_ref(slab: Handle, handle: Handle) -> i8 {
    let slab = <ForeignSlab as SlabAlloc<crate::UniFfiTag>>::get_clone(slab);
    match slab.remove(handle) {
        Ok((_, true)) => 1,
        Ok((_, false)) => 0,
        Err(e) => {
            println!("{e}");
            -1
        }
    }
}
