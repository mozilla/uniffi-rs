/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add a `self_type` field to type definitions, this way they get access to the fields in
//! `TypeNode`.

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    let namespace_name = namespace.name.clone();
    namespace.visit_mut(|rec: &mut Record| {
        rec.self_type = TypeNode {
            ty: Type::Record {
                namespace: namespace_name.clone(),
                name: rec.name.clone(),
            },
            ..TypeNode::default()
        }
    });
    namespace.visit_mut(|en: &mut Enum| {
        en.self_type = TypeNode {
            ty: Type::Enum {
                namespace: namespace_name.clone(),
                name: en.name.clone(),
            },
            ..TypeNode::default()
        };
    });
    namespace.visit_mut(|int: &mut Interface| {
        int.self_type = TypeNode {
            ty: Type::Interface {
                namespace: namespace_name.clone(),
                name: int.name.clone(),
                imp: int.imp.clone(),
            },
            ..TypeNode::default()
        };
    });
    namespace.visit_mut(|cbi: &mut CallbackInterface| {
        cbi.self_type = TypeNode {
            ty: Type::CallbackInterface {
                namespace: namespace_name.clone(),
                name: cbi.name.clone(),
            },
            ..TypeNode::default()
        };
    });
    namespace.visit_mut(|custom: &mut CustomType| {
        custom.self_type = TypeNode {
            ty: Type::Custom {
                namespace: namespace_name.clone(),
                name: custom.name.clone(),
                builtin: Box::new(custom.builtin.ty.clone()),
            },
            ..TypeNode::default()
        };
    });
    Ok(())
}
