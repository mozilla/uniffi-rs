/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Any types "missing" because the metadata didn't capture them.

use super::*;
use crate::anyhow;
use std::collections::HashMap;

pub fn pass(root: &mut Root) -> Result<()> {
    // ObjectTraitImpl carries only a trait name, we turn the trait into a real Type.
    // find out which module defines which traits.
    let mut known_traits: HashMap<String, String> = Default::default();
    root.visit(|m: &Module| {
        m.visit(|int: &Interface| {
            if matches!(int.imp, ObjectImpl::Trait | ObjectImpl::CallbackTrait) {
                known_traits.insert(int.name.clone(), m.name.to_owned());
            }
        });
        m.visit(|ci: &CallbackInterface| {
            known_traits.insert(ci.name.clone(), m.name.to_owned());
        });
    });
    // now fixup the impls.
    root.try_visit_mut(|oti: &mut ObjectTraitImpl| {
        let module_name = known_traits
            .get(&oti.trait_name)
            .ok_or_else(|| {
                anyhow!(
                    "object '{:?}' implements a trait '{}' but the trait can't be found",
                    oti.ty,
                    oti.trait_name
                )
            })?
            .to_string();
        oti.trait_ty = TypeNode {
            ty: Type::Interface {
                module_name,
                name: oti.trait_name.clone(),
                imp: ObjectImpl::Trait,
            },
            canonical_name: Default::default(),
            is_used_as_error: false,
            ffi_type: Default::default(),
        };
        Ok(())
    })
}
