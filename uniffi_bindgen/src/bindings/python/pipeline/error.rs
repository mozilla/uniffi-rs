/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn is_from_interface(throws_ty: &general::ThrowsType) -> bool {
    match &throws_ty.ty {
        None => false,
        Some(tn) => is_from_interface_inner(&tn.ty),
    }
}

fn is_from_interface_inner(ty: &Type) -> bool {
    match ty {
        // normalize the custom type to its builtin type
        Type::Custom { builtin, .. } => is_from_interface_inner(builtin),
        Type::Interface { .. } => true,
        _ => false,
    }
}
