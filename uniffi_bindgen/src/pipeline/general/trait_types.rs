/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! We are currently unable to capture full type info in the metadata for
//! ObjectTraitImpl. This finds a trait name and turns it into a type.
//! This imposes a constraint that trait names must be unique, which isn't ideal.

use super::*;
use crate::anyhow;
use std::collections::HashMap;

pub fn pass(root: &mut Root) -> Result<()> {
    // find out which namespace defines which traits.
    // tho unlikely, we handle that trait names might not be unique.
    // We only enforce trait name uniqueness when actually needed (ie, an ObjectTraitImpl)
    let mut known_traits: HashMap<String, Vec<String>> = HashMap::new();
    root.visit(|ns: &Namespace| {
        let mut note_trait = |tn: &str| {
            match known_traits.get_mut(tn) {
                Some(k) => k.push(ns.name.to_owned()),
                None => {
                    known_traits.insert(tn.to_owned(), vec![ns.name.to_owned()]);
                }
            };
        };
        ns.visit(|int: &Interface| {
            if matches!(int.imp, ObjectImpl::Trait | ObjectImpl::CallbackTrait) {
                note_trait(&int.name);
            };
        });
        ns.visit(|ci: &CallbackInterface| note_trait(&ci.name));
    });
    // now fixup the impls.
    root.try_visit_mut(|oti: &mut ObjectTraitImpl| {
        let namespace_names = known_traits.get(&oti.trait_name).ok_or_else(|| {
            anyhow!(
                "object '{:?}' implements a trait '{}' but the trait can't be found",
                oti.ty,
                oti.trait_name
            )
        })?;
        let namespace = match namespace_names.len() {
            1 => namespace_names[0].clone(),
            _ => bail!(
                "object '{:?}' implements a trait '{}' but the trait isn't unique: {:?}",
                oti.ty,
                oti.trait_name,
                namespace_names,
            ),
        };
        oti.trait_ty = TypeNode {
            ty: Type::Interface {
                namespace,
                name: oti.trait_name.clone(),
                imp: ObjectImpl::Trait,
            },
            ..Default::default()
        };
        Ok(())
    })
}
