/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

pub use super::*;

pub fn map_namespace(namespace: general::Namespace, context: &Context) -> Result<Module> {
    let mut child_context = context.clone();
    let context = &mut child_context;
    context.update_from_namespace(&namespace)?;

    let mut module = Module {
        cdylib_name: context.cdylib()?,
        has_async_fns: namespace.has_descendant(|callable: &general::Callable| callable.is_async()),
        has_callback_interface: namespace.has_descendant(|_: &general::CallbackInterface| true),
        has_async_callback_method: has_async_callback_method(&namespace),
        imports: module_imports(&namespace, context.config()?),
        exported_names: vec![],
        name: namespace.name.map_node(context)?,
        crate_name: namespace.crate_name.map_node(context)?,
        docstring: namespace.docstring.map_node(context)?,
        functions: namespace.functions.map_node(context)?,
        type_definitions: namespace.type_definitions.map_node(context)?,
        ffi_definitions: namespace.ffi_definitions.map_node(context)?,
        checksums: namespace.checksums.map_node(context)?,
        ffi_rustbuffer_alloc: namespace.ffi_rustbuffer_alloc,
        ffi_rustbuffer_from_bytes: namespace.ffi_rustbuffer_from_bytes,
        ffi_rustbuffer_free: namespace.ffi_rustbuffer_free,
        ffi_rustbuffer_reserve: namespace.ffi_rustbuffer_reserve,
        ffi_uniffi_contract_version: namespace.ffi_uniffi_contract_version,
        correct_contract_version: namespace.correct_contract_version,
        string_type_node: namespace.string_type_node.map_node(context)?,
    };
    // Generate exported names after mapping everything else.  This way we're sure all the renames
    // have taken effect.
    let mut exported_names = vec!["InternalError".to_string()];
    module.visit(|e: &Enum| exported_names.push(e.name.clone()));
    module.visit(|r: &Record| exported_names.push(r.name.clone()));
    module.visit(|f: &Function| exported_names.push(f.callable.name.clone()));
    module.visit(|i: &Interface| {
        exported_names.push(i.name.clone());
        exported_names.push(i.protocol.name.clone());
    });
    module.visit(|c: &CallbackInterface| exported_names.push(c.protocol.name.clone()));
    module.exported_names = exported_names;
    Ok(module)
}

fn has_async_callback_method(namespace: &general::Namespace) -> bool {
    let callback_interface_async = namespace.has_descendant(|cbi: &general::CallbackInterface| {
        cbi.has_descendant(|callable: &general::Callable| callable.is_async())
    });
    let trait_interface_async = namespace.has_descendant(|int: &general::Interface| {
        int.imp.has_callback_interface()
            && int.has_descendant(|callable: &general::Callable| callable.is_async())
    });
    callback_interface_async || trait_interface_async
}

fn module_imports(namespace: &general::Namespace, config: &PythonConfig) -> Vec<String> {
    let config_imports = config
        .custom_types
        .values()
        .flat_map(|custom_type_config| custom_type_config.imports.iter().flatten().cloned());

    let mut type_namespaces = HashSet::<String>::default();
    namespace.visit(|ty: &Type| {
        if let Some(namespace) = ty.namespace() {
            type_namespaces.insert(namespace.to_string());
        }
    });
    // Don't try to import the current module
    type_namespaces.remove(&namespace.name);

    let external_packages_imports = type_namespaces.iter().map(|namespace| {
        match config.external_packages.get(namespace.as_str()) {
            // No configuration, use the module name as a relative import
            None => format!(".{namespace}"),
            // Empty string, use the module name as an absolute import
            Some(value) if value.is_empty() => namespace.to_string(),
            // Package name for configuration, use that name
            Some(package_name) => package_name.clone(),
        }
    });

    config_imports.chain(external_packages_imports).collect()
}
