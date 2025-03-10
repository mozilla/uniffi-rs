/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! TypeNode::is_used_as_error field
//!
//! Bindings often treat error types differently, for example by adding `Exception` to their names.

use indexmap::IndexSet;

use super::*;

pub fn step(module: &mut Module) -> Result<()> {
    let mut used_as_error = IndexSet::<Type>::default();
    module.visit(|callable: &Callable| {
        if let Some(ty) = &callable.throws_type.ty {
            used_as_error.insert(ty.ty.clone());
        }
    });
    module.visit_mut(|ty: &mut TypeNode| {
        ty.is_used_as_error = used_as_error.contains(&ty.ty);
    });
    Ok(())
}
