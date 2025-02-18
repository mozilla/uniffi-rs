/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Extract common data from Function/Method/Constructor into Callable

use super::*;

pub fn pass(root: &mut Root) -> Result<()> {
    root.visit_mut(|func: &mut Function| {
        func.callable = Callable! {
            name: func.name.clone(),
            is_async: func.is_async,
            kind: CallableKind::Function,
            // Take data from the vecs since these fields are being removed from the pass anyways.
            arguments: func.inputs.take(),
            return_type: ReturnType! {
                ty: func.return_type.take().map(|ty| TypeNode! { ty }),
            },
            throws_type: ThrowsType! {
                ty: func.throws.take().map(|ty| TypeNode! { ty }),
            },
            checksum: func.checksum,
        }
    });
    root.visit_mut(|module: &mut Module| {
        let module_name = module.name.clone();
        module.visit_mut(|int: &mut Interface| {
            let interface_name = int.name.clone();
            let interface_imp = int.imp.clone();
            int.visit_mut(|cons: &mut Constructor| {
                cons.callable = Callable! {
                    name: cons.name.clone(),
                    is_async: cons.is_async,
                    kind: CallableKind_Constructor! {
                        interface_name: interface_name.clone(),
                        primary: cons.name == "new",
                    },
                    // Take data from the vecs since these fields are being removed from the pass anyways.
                    arguments: cons.inputs.take(),
                    return_type: ReturnType! {
                        ty: Some(TypeNode! {
                            ty: Type_Interface! {
                                module_name: module_name.clone(),
                                name: interface_name.clone(),
                                imp: interface_imp.clone(),
                            },
                        }),
                    },
                    throws_type: ThrowsType! {
                        ty: cons.throws.take().map(|ty| TypeNode! { ty }),
                    },
                    checksum: cons.checksum,
                }
            });
            int.visit_mut(|meth: &mut Method| {
                meth.callable = Callable! {
                    name: meth.name.clone(),
                    is_async: meth.is_async,
                    kind: CallableKind_Method! {
                        interface_name: interface_name.clone(),
                    },
                    arguments: meth.inputs.take(),
                    return_type: ReturnType! {
                        ty: meth.return_type.take().map(|ty| TypeNode! { ty }),
                    },
                    throws_type: ThrowsType! {
                        ty: meth.throws.take().map(|ty| TypeNode! { ty }),
                    },
                    checksum: meth.checksum,
                }
            });
        });
    });
    root.visit_mut(|cbi: &mut CallbackInterface| {
        let interface_name = cbi.name.clone();
        cbi.visit_mut(|meth: &mut Method| {
            meth.callable = Callable! {
                name: meth.name.clone(),
                is_async: meth.is_async,
                kind: CallableKind_Method! {
                    interface_name: interface_name.clone(),
                },
                arguments: meth.inputs.take(),
                return_type: ReturnType! {
                    ty: meth.return_type.take().map(|ty| TypeNode! { ty })
                },
                throws_type: ThrowsType! {
                    ty: meth.throws.take().map(|ty| TypeNode! { ty })
                },
                checksum: meth.checksum,
            }
        });
    });
    Ok(())
}
