/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Removes uniffi_traits from nodes, moving them to uniffi_trait_methods.

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    namespace.visit_mut(|i: &mut Interface| {
        consume_from(&mut i.uniffi_trait_methods, &mut i.uniffi_traits);
    });
    namespace.visit_mut(|e: &mut Enum| {
        consume_from(&mut e.uniffi_trait_methods, &mut e.uniffi_traits);
    });
    namespace.visit_mut(|rec: &mut Record| {
        consume_from(&mut rec.uniffi_trait_methods, &mut rec.uniffi_traits);
    });
    Ok(())
}

fn consume_from(dest: &mut UniffiTraitMethods, uniffi_traits: &mut Vec<UniffiTrait>) {
    for t in uniffi_traits.drain(..) {
        match t {
            UniffiTrait::Debug { fmt } => dest.debug_fmt = Some(fmt),
            UniffiTrait::Display { fmt } => dest.display_fmt = Some(fmt),
            UniffiTrait::Eq { eq, ne } => {
                dest.eq_eq = Some(eq);
                dest.eq_ne = Some(ne);
            }
            UniffiTrait::Hash { hash } => dest.hash_hash = Some(hash),
            UniffiTrait::Ord { cmp } => dest.ord_cmp = Some(cmp),
        }
    }
}
