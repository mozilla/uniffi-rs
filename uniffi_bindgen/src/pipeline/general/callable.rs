/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extract common data from Function/Method/Constructor into Callable

use super::*;

fn new_callable_method(meth: &Method, self_type: TypeNode) -> Callable {
    Callable {
        name: meth.name.clone(),
        is_async: meth.is_async,
        kind: CallableKind::Method { self_type },
        arguments: meth.inputs.clone(),
        return_type: ReturnType {
            ty: meth.return_type.clone().map(|ty| TypeNode {
                ty,
                ..TypeNode::default()
            }),
        },
        throws_type: ThrowsType {
            ty: meth.throws.clone().map(|ty| TypeNode {
                ty,
                ..TypeNode::default()
            }),
        },
        checksum: meth.checksum,
        ..Callable::default()
    }
}

pub fn pass(root: &mut Root) -> Result<()> {
    root.visit_mut(|func: &mut Function| {
        func.callable = Callable {
            name: func.name.clone(),
            is_async: func.is_async,
            kind: CallableKind::Function,
            arguments: func.inputs.clone(),
            return_type: ReturnType {
                ty: func.return_type.clone().map(|ty| TypeNode {
                    ty,
                    ..TypeNode::default()
                }),
            },
            throws_type: ThrowsType {
                ty: func.throws.clone().map(|ty| TypeNode {
                    ty,
                    ..TypeNode::default()
                }),
            },
            checksum: func.checksum,
            ..Callable::default()
        }
    });
    root.visit_mut(|namespace: &mut Namespace| {
        let namespace_name = namespace.name.clone();
        namespace.visit_mut(|int: &mut Interface| {
            let interface_name = int.name.clone();
            let interface_imp = int.imp.clone();
            let self_type = int.self_type.clone();
            int.visit_mut(|cons: &mut Constructor| {
                cons.callable = Callable {
                    name: cons.name.clone(),
                    is_async: cons.is_async,
                    kind: CallableKind::Constructor {
                        interface_name: interface_name.clone(),
                        primary: cons.name == "new",
                    },
                    arguments: cons.inputs.clone(),
                    return_type: ReturnType {
                        ty: Some(TypeNode {
                            ty: Type::Interface {
                                namespace: namespace_name.clone(),
                                name: interface_name.clone(),
                                imp: interface_imp.clone(),
                            },
                            ..TypeNode::default()
                        }),
                    },
                    throws_type: ThrowsType {
                        ty: cons.throws.clone().map(|ty| TypeNode {
                            ty,
                            ..TypeNode::default()
                        }),
                    },
                    checksum: cons.checksum,
                    ..Callable::default()
                }
            });
            int.visit_mut(|meth: &mut Method| {
                meth.callable = new_callable_method(meth, self_type.clone());
            });
        });
    });
    root.visit_mut(|cbi: &mut CallbackInterface| {
        let self_type = cbi.self_type.clone();

        cbi.visit_mut(|m: &mut Method| {
            m.callable = new_callable_method(m, self_type.clone());
        })
    });

    root.visit_mut(|e: &mut Enum| {
        let self_type = e.self_type.clone();
        e.visit_mut(|meth: &mut Method| {
            meth.callable = new_callable_method(meth, self_type.clone());
        });
    });

    root.visit_mut(|r: &mut Record| {
        let self_type = r.self_type.clone();
        r.visit_mut(|meth: &mut Method| {
            meth.callable = new_callable_method(meth, self_type.clone());
        });
    });
    Ok(())
}
