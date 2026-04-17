/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_namespace(input: general::Namespace, context: &Context) -> Result<Package> {
    let mut context = context.clone();
    context.update_from_namespace(&input);

    let config = context.config()?.clone();
    let imports = config
        .custom_types
        .values()
        .flat_map(|c| c.imports.clone())
        .flatten()
        .collect();

    // Map everything except scaffolding_functions
    let package = Package {
        name: context.package_name(&input.name)?.to_string(),
        crate_name: input.name,
        config,
        functions: input.functions.map_node(&context)?,
        type_definitions: map_type_definitions(input.type_definitions, &context)?,
        scaffolding_functions: vec![],
        imports,
    };
    // Collect scaffolding_functions
    let scaffolding_functions = package
        .functions
        .iter()
        .map(|f| {
            (
                &f.jni_method_name,
                &f.callable,
                ScaffoldingFunctionKind::Function,
            )
        })
        .chain(package.classes().flat_map(|c| {
            c.methods
                .iter()
                .map(|m| {
                    (
                        &m.jni_method_name,
                        &m.callable,
                        ScaffoldingFunctionKind::Method,
                    )
                })
                .chain(c.constructors.iter().map(|c| {
                    (
                        &c.jni_method_name,
                        &c.callable,
                        ScaffoldingFunctionKind::Function,
                    )
                }))
        }))
        .map(|(name, callable, kind)| ScaffoldingFunction {
            jni_method_name: name.clone(),
            callable: callable.clone(),
            kind,
        })
        .chain(
            package
                .uniffi_trait_methods()
                .flat_map(UniffiTraitMethods::scaffolding_functions),
        )
        .collect();

    // Put everything together
    Ok(Package {
        scaffolding_functions,
        ..package
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
            general::TypeDefinition::Interface(int) => {
                mapped.push(TypeDefinition::Interface(interfaces::map_interface(
                    &int, context,
                )?));
                mapped.push(TypeDefinition::Class(int.map_node(context)?));
            }
            general::TypeDefinition::CallbackInterface(cbi) => {
                mapped.push(TypeDefinition::Interface(
                    callbacks::interface_for_callback_interface(&cbi, context)?,
                ));
                mapped.push(TypeDefinition::CallbackInterface(cbi.map_node(context)?));
            }
            general::TypeDefinition::Custom(c) => {
                mapped.push(TypeDefinition::Custom(c.map_node(context)?));
            }
            general::TypeDefinition::Box(inner) => {
                mapped.push(TypeDefinition::Box(inner.map_node(context)?));
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
            general::TypeDefinition::Simple(inner) => match &inner.ty {
                Type::Duration => mapped.push(TypeDefinition::Duration(inner.map_node(context)?)),
                Type::Timestamp => mapped.push(TypeDefinition::Timestamp(inner.map_node(context)?)),
                Type::Bytes => mapped.push(TypeDefinition::Bytes(inner.map_node(context)?)),
                _ => (),
            },
            _ => (),
        }
    }
    Ok(mapped)
}
