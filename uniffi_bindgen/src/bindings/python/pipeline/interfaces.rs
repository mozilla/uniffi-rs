/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(int: &mut Interface) -> Result<()> {
    match &int.imp {
        ObjectImpl::Struct | ObjectImpl::Trait => {
            // Interface that's only implemented in Rust:
            //   - Give the interface the main name and append the `Protocol` suffix to the protocol
            //   - Make the protocol inherit from `typing.Protocol`, since the #2264 doesn't affect
            //     these interfaces
            int.protocol = Protocol {
                name: format!("{}Protocol", int.name),
                base_classes: vec!["typing.Protocol".to_string()],
                methods: int.methods.clone(),
                docstring: int.docstring.clone(),
            };
        }
        ObjectImpl::CallbackTrait => {
            // Trait interface that can be implemented in Rust or Python
            //   - Give the protocol the main name and append the `Impl` suffix to the interface
            //   - Don't make the protocol inherit from `typing.Protocol`.  We're going to inherit
            //     from it so it's not a typical Python protocol
            //     (http://github.com/mozilla/uniffi-rs/issues/2264).
            int.protocol = Protocol {
                name: int.name.clone(),
                base_classes: vec![],
                methods: int.methods.clone(),
                docstring: int.docstring.clone(),
            };
            int.name = format!("{}Impl", int.name);
        }
    };

    int.base_classes.push(int.protocol.name.clone());
    if int.self_type.is_used_as_error {
        int.base_classes.push("Exception".to_string());
    }
    for t in int.trait_impls.iter() {
        let (name, external_package_name) = match &t.trait_ty.ty {
            Type::Interface {
                name,
                external_package_name,
                ..
            } => (name, external_package_name),
            Type::CallbackInterface {
                name,
                external_package_name,
                ..
            } => (name, external_package_name),
            _ => bail!("trait_ty {:?} isn't a trait", t),
        };
        let fq = match external_package_name {
            None => name.clone(),
            Some(package) => format!("{package}.{name}"),
        };
        int.base_classes.push(fq);
    }
    int.has_primary_constructor = int.has_descendant(|c: &Callable| c.is_primary_constructor());
    Ok(())
}
