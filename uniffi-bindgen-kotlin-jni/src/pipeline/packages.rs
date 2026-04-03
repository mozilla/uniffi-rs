/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_namespace(input: general::Namespace, context: &Context) -> Result<Package> {
    let mut context = context.clone();
    context.update_from_namespace(&input);

    Ok(Package {
        name: context.package_name_for_namespace(&input.name)?.to_string(),
        crate_name: input.name,
        config: context.config()?.clone(),
        functions: input.functions.map_node(&context)?,
        type_definitions: map_type_definitions(input.type_definitions, &context)?,
    })
}

pub fn map_type_definitions(
    type_defs: Vec<general::TypeDefinition>,
    context: &Context,
) -> Result<Vec<TypeDefinition>> {
    let mut mapped = vec![];
    for type_def in type_defs {
        match type_def {
            general::TypeDefinition::Record(rec) => {
                mapped.push(TypeDefinition::Record(rec.map_node(context)?));
            }
            general::TypeDefinition::Enum(en) => {
                mapped.push(TypeDefinition::Enum(en.map_node(context)?));
            }
            general::TypeDefinition::Interface(int) => match int.imp {
                ObjectImpl::Struct => {
                    mapped.push(TypeDefinition::Interface(interfaces::map_interface(
                        &int, context,
                    )?));
                    mapped.push(TypeDefinition::Class(int.map_node(context)?));
                }
                ObjectImpl::Trait(_) => todo!(),
            },
            general::TypeDefinition::Optional(opt) => {
                mapped.push(TypeDefinition::Optional(opt.map_node(context)?));
            }
            general::TypeDefinition::Sequence(seq) => {
                mapped.push(TypeDefinition::Sequence(seq.map_node(context)?));
            }
            general::TypeDefinition::Map(map) => {
                mapped.push(TypeDefinition::Map(map.map_node(context)?));
            }
            general::TypeDefinition::Set(set) => {
                mapped.push(TypeDefinition::Set(set.map_node(context)?));
            }
            // All other variants are still TODO
            _ => (),
        }
    }
    Ok(mapped)
}

impl Package {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.crate_name)
    }

    /// JNI methods to define/export from Rust
    ///
    /// Generates a list of (jni_method_name, callable) pairs.
    pub fn jni_methods(&self) -> impl Iterator<Item = (&str, &Callable)> {
        let functions = self
            .functions
            .iter()
            .map(|f| (f.jni_method_name.as_str(), &f.callable));
        let methods = self.classes().flat_map(|c| {
            c.methods
                .iter()
                .map(|m| (m.jni_method_name.as_str(), &m.callable))
        });
        let constructors = self.classes().flat_map(|c| {
            c.constructors
                .iter()
                .map(|c| (c.jni_method_name.as_str(), &c.callable))
        });

        functions.chain(methods).chain(constructors)
    }

    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Class(cls) => Some(cls),
                _ => None,
            })
    }
}
