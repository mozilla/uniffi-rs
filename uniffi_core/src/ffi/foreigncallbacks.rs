/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains code to handle foreign callbacks - C-ABI functions that are defined by a
//! foreign language, then registered with UniFFI.  These callbacks are used to implement callback
//! interfaces, async scheduling etc. Foreign callbacks are registered at startup, when the foreign
//! code loads the exported library. For each callback type, we also define a "cell" type for
//! storing the callback.

use std::{
    ptr::NonNull,
    sync::{PoisonError, RwLock},
};

/// Bit position of the realm identifier inside a foreign handle.
///
/// A realm is one instance of a foreign runtime: a Node worker thread, a JVM classloader, a
/// Python sub-interpreter. Several can load the same cdylib in one process, and each registers
/// its own callback vtable pointing at its own runtime.
///
/// Foreign bindings reserve only the lowest bit of a handle, which must be set so a foreign
/// handle is distinguishable from a Rust pointer. The top bits are free, so a realm tags every
/// handle it mints and the tag travels back into Rust with the handle. This split leaves each
/// realm 2^48 handles and allows 2^16 realms.
pub const UNIFFI_REALM_SHIFT: u32 = 48;

/// Identifier of the realm that minted `handle`.
///
/// An untagged handle yields 0, so bindings that do not tag keep working unchanged: they
/// register into realm 0 and read back from realm 0.
pub const fn uniffi_realm_of_handle(handle: u64) -> u64 {
    handle >> UNIFFI_REALM_SHIFT
}

/// Cell type that stores one `NonNull<T>` per realm.
///
/// Registration happens once per realm at startup, so the realm list is tiny and effectively
/// write-once. A linear scan of it costs less than hashing, and is dwarfed in any case by the
/// foreign call that every lookup precedes.
#[doc(hidden)]
pub struct UniffiForeignPointerCell<T>(RwLock<Vec<(u64, NonNull<T>)>>);

impl<T> UniffiForeignPointerCell<T> {
    pub const fn new() -> Self {
        Self(RwLock::new(Vec::new()))
    }

    /// Store the pointer registered by `realm`, replacing any previous registration for it.
    pub fn set_for_realm(&self, realm: u64, callback: NonNull<T>) {
        let mut realms = self.0.write().unwrap_or_else(PoisonError::into_inner);
        match realms.iter_mut().find(|(id, _)| *id == realm) {
            Some((_, registered)) => *registered = callback,
            None => realms.push((realm, callback)),
        }
    }

    pub fn set(&self, callback: NonNull<T>) {
        self.set_for_realm(0, callback);
    }

    /// Look up the pointer registered by the realm that minted `handle`.
    ///
    /// Falls back to realm 0 when that realm registered nothing, which is what a handle minted
    /// by bindings that do not tag looks like.
    pub fn get_for_handle(&self, handle: u64) -> &T {
        let realm = uniffi_realm_of_handle(handle);
        let realms = self.0.read().unwrap_or_else(PoisonError::into_inner);
        let registered = realms
            .iter()
            .find(|(id, _)| *id == realm)
            .or_else(|| realms.iter().find(|(id, _)| *id == 0))
            .map(|(_, registered)| *registered);
        drop(realms);

        // The pointer is owned by the foreign bindings and outlives every call into Rust, which
        // is the same contract the single-pointer cell relied on.
        unsafe {
            registered
                .expect("Foreign pointer not set.  This is likely a uniffi bug.")
                .as_ref()
        }
    }

    pub fn get(&self) -> &T {
        self.get_for_handle(0)
    }
}

impl<T> Default for UniffiForeignPointerCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Send for UniffiForeignPointerCell<T> {}
unsafe impl<T> Sync for UniffiForeignPointerCell<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle_in(realm: u64, n: u64) -> u64 {
        (realm << UNIFFI_REALM_SHIFT) | (n << 1) | 1
    }

    #[test]
    fn realm_is_read_from_the_top_bits_and_untagged_means_realm_zero() {
        assert_eq!(uniffi_realm_of_handle(1), 0);
        assert_eq!(uniffi_realm_of_handle(handle_in(0, 7)), 0);
        assert_eq!(uniffi_realm_of_handle(handle_in(3, 7)), 3);
        // The realm is whatever the top bits hold, so nothing needs clamping.
        assert_eq!(
            uniffi_realm_of_handle(u64::MAX),
            u64::MAX >> UNIFFI_REALM_SHIFT
        );
    }

    #[test]
    fn each_realm_reads_back_its_own_registration() {
        let (mut a, mut b) = (1u32, 2u32);
        let cell = UniffiForeignPointerCell::new();

        cell.set_for_realm(0, NonNull::from(&mut a));
        cell.set_for_realm(3, NonNull::from(&mut b));

        // The registrations do not overwrite each other, which is the whole point.
        assert_eq!(*cell.get_for_handle(handle_in(0, 1)), 1);
        assert_eq!(*cell.get_for_handle(handle_in(3, 1)), 2);
    }

    #[test]
    fn re_registering_a_realm_replaces_it_without_growing_the_list() {
        let (mut a, mut b) = (1u32, 2u32);
        let cell = UniffiForeignPointerCell::new();

        cell.set_for_realm(5, NonNull::from(&mut a));
        cell.set_for_realm(5, NonNull::from(&mut b));

        assert_eq!(*cell.get_for_handle(handle_in(5, 1)), 2);
        assert_eq!(cell.0.read().unwrap().len(), 1);
    }

    #[test]
    fn a_realm_that_registered_nothing_falls_back_to_realm_zero() {
        let mut a = 1u32;
        let cell = UniffiForeignPointerCell::new();
        cell.set(NonNull::from(&mut a));

        assert_eq!(*cell.get_for_handle(handle_in(9, 1)), 1);
        assert_eq!(*cell.get(), 1);
    }

    #[test]
    fn a_realm_beyond_any_fixed_bound_still_registers() {
        let mut a = 7u32;
        let cell = UniffiForeignPointerCell::new();
        let realm = u64::MAX >> UNIFFI_REALM_SHIFT;

        cell.set_for_realm(realm, NonNull::from(&mut a));

        assert_eq!(*cell.get_for_handle(handle_in(realm, 1)), 7);
    }
}
