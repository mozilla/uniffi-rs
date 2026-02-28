/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn protocol(int: &general::Interface, context: &Context) -> Result<Protocol> {
    Ok(match &int.imp {
        ObjectImpl::Struct | ObjectImpl::Trait => {
            // Interface that's only implemented in Rust:
            //   - Give the interface the main name and append the `Protocol` suffix to the protocol
            //   - Make the protocol inherit from `typing.Protocol`, since the #2264 doesn't affect
            //     these interfaces
            Protocol {
                name: format!("{}Protocol", names::type_name(&int.name)),
                base_classes: vec!["typing.Protocol".to_string()],
                methods: int.methods.clone().map_node(context)?,
                docstring: int.docstring.clone(),
            }
        }
        ObjectImpl::CallbackTrait => {
            // Trait interface that can be implemented in Rust or Python
            //   - Give the protocol the main name and append the `Impl` suffix to the interface
            //   - Don't make the protocol inherit from `typing.Protocol`.  We're going to inherit
            //     from it so it's not a typical Python protocol
            //     (http://github.com/mozilla/uniffi-rs/issues/2264).
            Protocol {
                name: names::type_name(&int.name),
                base_classes: vec![],
                methods: int.methods.clone().map_node(context)?,
                docstring: int.docstring.clone(),
            }
        }
    })
}

pub fn name(int: &general::Interface) -> String {
    // Interface name, see `protocol` for a discussion of the logic here
    match &int.imp {
        ObjectImpl::Struct | ObjectImpl::Trait => names::type_name(&int.name),
        ObjectImpl::CallbackTrait => names::type_name(&format!("{}Impl", int.name)),
    }
}

pub fn base_classes(int: &general::Interface, context: &Context) -> Result<Vec<String>> {
    let mut base_classes = vec![];

    base_classes.push(protocol(int, context)?.name);
    if int.self_type.is_used_as_error {
        base_classes.push("Exception".to_string());
    }
    for t in int.trait_impls.iter() {
        let (name, namespace) = match &t.trait_ty.ty {
            Type::Interface {
                name,
                namespace,
                imp,
                ..
            } => {
                // For trait interfaces implement in Rust-only, the protocol has `Protocol` appended.
                // Trait interfaces with foreign implementations don't have that
                match imp {
                    ObjectImpl::Trait => (format!("{name}Protocol"), namespace),
                    ObjectImpl::CallbackTrait => (name.to_string(), namespace),
                    ObjectImpl::Struct => {
                        bail!("Objects can only inherit from traits, not other objects")
                    }
                }
            }

            Type::CallbackInterface {
                name, namespace, ..
            } => (name.to_string(), namespace),
            _ => bail!("trait_ty {:?} isn't a trait", t),
        };
        let name = names::type_name(&name);
        let fq = match context.external_package_name(namespace)? {
            None => name.clone(),
            Some(package) => format!("{package}.{name}"),
        };
        base_classes.push(fq);
    }
    Ok(base_classes)
}

pub fn map_constructors(
    interface_name: &str,
    constructors: Vec<general::Constructor>,
    context: &Context,
) -> Result<Vec<Constructor>> {
    constructors
        .into_iter()
        .map(|c| {
            if c.callable.is_primary_constructor() && c.callable.is_async() {
                bail!("Async primary constructors not supported but {interface_name} has one");
            }
            c.map_node(context)
        })
        .collect()
}
