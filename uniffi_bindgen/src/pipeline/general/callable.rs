/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extract common data from Function/Method/Constructor into Callable

use super::*;

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
    root.visit_mut(|module: &mut Module| {
        let module_name = module.name.clone();
        module.visit_mut(|int: &mut Interface| {
            let interface_name = int.name.clone();
            let interface_imp = int.imp.clone();
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
                                module_name: module_name.clone(),
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
                meth.callable = Callable {
                    name: meth.name.clone(),
                    is_async: meth.is_async,
                    kind: CallableKind::Method {
                        interface_name: interface_name.clone(),
                    },
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
            });
        });
    });
    root.visit_mut(|cbi: &mut CallbackInterface| {
        let interface_name = cbi.name.clone();
        cbi.visit_mut(|meth: &mut Method| {
            meth.callable = Callable {
                name: meth.name.clone(),
                is_async: meth.is_async,
                kind: CallableKind::Method {
                    interface_name: interface_name.clone(),
                },
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
        });
    });
    Ok(())
}
