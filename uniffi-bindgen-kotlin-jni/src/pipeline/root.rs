/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_bindgen::pipeline::general::sort::sort_type_definitions_ignore_container_children;

use super::*;

pub fn map_root(mut input: general::Root, context: &Context) -> Result<Root> {
    let mut context = context.clone();

    // Re-sort the type definitions with sort_type_definitions_ignore_container_children.
    //
    // It allows us to handle recursive types when calculating FFI layouts, since:
    //   - box/vec/set have static FFI layouts, we don't care about the children
    //   - Rust requires that one field is a box/vec/set when types are recursive.
    //
    //  This must happen before the `update_from_root` call.
    for namespace in input.namespaces.values_mut() {
        namespace.type_definitions =
            sort_type_definitions_ignore_container_children(namespace.type_definitions.drain(..));
    }
    context.update_from_root(&input)?;

    // Find type definitions for builtin types,
    // these get defined in the `uniffi` package
    let mut non_user_type_definitions = IndexMap::new();
    for namespace in input.namespaces.values_mut() {
        for type_def in namespace.type_definitions.iter() {
            let type_node = type_def.type_node();
            if !type_node.ty.has_user_type() {
                non_user_type_definitions
                    .entry(type_node.id)
                    .or_insert_with(|| type_def.clone());
            }
        }
    }

    let packages = Vec::from_iter(input.namespaces.map_node(&context)?.into_values());

    Ok(Root {
        cdylib: input.cdylib,
        non_user_type_definitions: non_user_type_definitions
            .into_values()
            .collect::<Vec<_>>()
            .map_node(&context)?,
        packages,
    })
}

impl Root {
    pub fn cdylib_name(&self) -> Result<String> {
        let config_names: IndexSet<_> = self
            .packages
            .iter()
            .filter_map(|p| p.config.cdylib_name.as_deref())
            .collect();
        Ok(match config_names.len() {
            0 => match &self.cdylib {
                Some(name) => name.to_string(),
                None => bail!("Unknown cdylib name.  Use `src:[crate_name]` to generate bindings or set it in a `uniffi.toml` config"),
            }
            1 => config_names.into_iter().next().unwrap().to_string(),
            _ => bail!("Conflicting cdylib names in `uniffi.toml` files: {:?}", Vec::from_iter(config_names)),
        })
    }

    pub fn unique_type_definitions(&self) -> impl Iterator<Item = &TypeDefinition> {
        let mut seen = HashSet::new();
        self.non_user_type_definitions
            .iter()
            .chain(self.packages.iter().flat_map(|p| &p.type_definitions))
            .filter(move |type_def| seen.insert(type_def.type_node().id))
    }

    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.packages.iter().flat_map(Package::classes)
    }

    pub fn boxes(&self) -> impl Iterator<Item = &BoxedType> {
        self.packages.iter().flat_map(Package::boxes)
    }

    pub fn disable_java_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        //
        // Maybe we can leverage https://github.com/mozilla/uniffi-rs/issues/2866 to improve this
        self.packages.iter().any(|p| p.config.disable_java_cleaner)
    }

    pub fn enable_android_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        //
        // Maybe we can leverage https://github.com/mozilla/uniffi-rs/issues/2866 to improve this
        self.packages.iter().any(|p| p.config.android_cleaner())
    }
}
