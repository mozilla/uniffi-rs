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
        self.functions
            .iter()
            .map(|f| (f.jni_method_name.as_str(), &f.callable))
    }
}
