/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add a `self_type` field to type definitions, this way they get access to the fields in
//! `TypeNode`.

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let module_name = module.name.clone();
    module.visit_mut(|rec: &mut Record| {
        rec.self_type = TypeNode! {
            ty: Type_Record! {
                module_name: module_name.clone(),
                name: rec.name.clone(),
            },
        }
    });
    module.visit_mut(|en: &mut Enum| {
        en.self_type = TypeNode! {
            ty: Type_Enum! {
                module_name: module_name.clone(),
                name: en.name.clone(),
            },
        };
    });
    module.visit_mut(|int: &mut Interface| {
        int.self_type = TypeNode! {
            ty: Type_Interface! {
                module_name: module_name.clone(),
                name: int.name.clone(),
                imp: int.imp.clone(),
            },
        };
    });
    module.visit_mut(|cbi: &mut CallbackInterface| {
        cbi.self_type = TypeNode! {
            ty: Type_CallbackInterface! {
                module_name: module_name.clone(),
                name: cbi.name.clone(),
            },
        };
    });
    module.visit_mut(|custom: &mut CustomType| {
        custom.self_type = TypeNode! {
            ty: Type_Custom! {
                module_name: module_name.clone(),
                name: custom.name.clone(),
                builtin: Box::new(custom.builtin.ty.clone()),
            },
        };
    });
    Ok(())
}
