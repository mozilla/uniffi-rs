/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_namespace(input: general::Namespace, context: &Context) -> Result<Package> {
    let mut context = context.clone();
    context.update_from_namespace(&input);

    Ok(Package {
        name: context.package_name(&input.name)?.to_string(),
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
            general::TypeDefinition::Optional(inner) => {
                mapped.push(TypeDefinition::Optional(inner.map_node(context)?));
            }
            general::TypeDefinition::Sequence(inner) => {
                mapped.push(TypeDefinition::Sequence(inner.map_node(context)?));
            }
            general::TypeDefinition::Map(inner) => {
                mapped.push(TypeDefinition::Map(inner.map_node(context)?));
            }
            // All other variants are still TODO
            _ => (),
        }
    }
    Ok(mapped)
}
