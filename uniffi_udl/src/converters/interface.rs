/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::APIConverter;
use crate::attributes::InterfaceAttributes;
use crate::{converters::convert_docstring, InterfaceCollector};
use anyhow::{bail, Result};
use std::collections::HashSet;
use uniffi_meta::{
    ConstructorMetadata, MethodMetadata, MethodReceiver, ObjectImpl, ObjectMetadata, Type,
};

impl APIConverter<ObjectMetadata> for weedle::InterfaceDefinition<'_> {
    fn convert(&self, ci: &mut InterfaceCollector) -> Result<ObjectMetadata> {
        if self.inheritance.is_some() {
            bail!("interface inheritance is not supported");
        }
        let attributes = match &self.attributes {
            Some(attrs) => InterfaceAttributes::try_from(attrs)?,
            None => Default::default(),
        };

        let object_name = self.identifier.0;
        let object_impl = attributes.object_impl()?;
        // Convert each member into a constructor or method, guarding against duplicate names.
        // They get added to the ci and aren't carried in ObjectMetadata.
        let mut member_names = HashSet::new();
        for member in &self.members.body {
            match member {
                weedle::interface::InterfaceMember::Constructor(t) => {
                    let mut cons: ConstructorMetadata = t.convert(ci)?;
                    if object_impl == ObjectImpl::Trait {
                        bail!(
                            "Trait interfaces can not have constructors: \"{}\"",
                            cons.name
                        )
                    }
                    if !member_names.insert(cons.name.clone()) {
                        bail!("Duplicate interface member name: \"{}\"", cons.name)
                    }
                    cons.self_name = object_name.to_string();
                    ci.items.insert(cons.into());
                }
                weedle::interface::InterfaceMember::Operation(t) => {
                    let mut method: MethodMetadata = t.convert(ci)?;
                    if !member_names.insert(method.name.clone()) {
                        bail!("Duplicate interface member name: \"{}\"", method.name)
                    }
                    // a little smelly that we need to fixup the receiver here, but it is what it is...
                    let new_name = object_name.to_string();
                    match method.receiver {
                        MethodReceiver::Enum { ref mut name, .. } => *name = new_name,
                        MethodReceiver::Record { ref mut name, .. } => *name = new_name,
                        MethodReceiver::Object { ref mut name, .. } => *name = new_name,
                    }
                    ci.items.insert(method.into());
                }
                _ => bail!("no support for interface member type {:?} yet", member),
            }
        }
        // Add uniffi-traits to the CI.
        let other = Type::Object {
            module_path: ci.module_path().to_string(),
            name: object_name.to_string(),
            imp: object_impl,
        };

        for ut in super::make_uniffi_traits(
            MethodReceiver::Object {
                module_path: ci.module_path().to_string(),
                name: object_name.to_string(),
            },
            &attributes.get_traits(),
            &other,
        )? {
            ci.items.insert(ut.into());
        }

        Ok(ObjectMetadata {
            module_path: ci.module_path(),
            name: object_name.to_string(),
            remote: attributes.contains_remote(),
            imp: object_impl,
            docstring: self.docstring.as_ref().map(|v| convert_docstring(&v.0)),
        })
    }
}
