/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_trait_vec(
    traits: Vec<initial::UniffiTrait>,
    context: &Context,
) -> Result<UniffiTraitMethods> {
    let mut dest = UniffiTraitMethods::default();
    for t in traits {
        match t {
            initial::UniffiTrait::Debug { fmt } => {
                dest.debug_fmt = Some(fmt.map_node(context)?);
            }
            initial::UniffiTrait::Display { fmt } => {
                dest.display_fmt = Some(fmt.map_node(context)?);
            }
            initial::UniffiTrait::Eq { eq, ne } => {
                dest.eq_eq = Some(eq.map_node(context)?);
                dest.eq_ne = Some(ne.map_node(context)?);
            }
            initial::UniffiTrait::Hash { hash } => {
                dest.hash_hash = Some(hash.map_node(context)?);
            }
            initial::UniffiTrait::Ord { cmp } => {
                dest.ord_cmp = Some(cmp.map_node(context)?);
            }
        }
    }
    Ok(dest)
}
