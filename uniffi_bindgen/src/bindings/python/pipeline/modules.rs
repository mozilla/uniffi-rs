/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
*
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use super::*;
use indexmap::IndexSet;

pub fn pass(root: &mut Root) -> Result<()> {
    let default_cdylib = root.cdylib.as_deref().unwrap_or("uniffi").to_string();
    root.try_visit_mut(|namespace: &mut Namespace| {
        namespace.cdylib_name = match &namespace.config.cdylib_name {
            Some(cdylib) => cdylib.clone(),
            None => default_cdylib.to_string(),
        };

        namespace.has_async_fns = namespace.has_descendant(|callable: &Callable| callable.is_async);
        namespace.has_callback_interface =
            namespace.has_descendant_with_type::<CallbackInterface>();
        namespace.has_async_callback_method = namespace.has_descendant(|vtable: &VTable| {
            vtable.has_descendant(|callable: &Callable| callable.is_async)
        });

        let mut module_imports = IndexSet::new();
        namespace.visit(|custom_type_config: &CustomTypeConfig| {
            if let Some(imports) = &custom_type_config.imports {
                module_imports.extend(imports.clone());
            }
        });
        namespace.imports.extend(module_imports);

        let mut exported_names = vec!["InternalError".to_string()];
        namespace.visit(|e: &Enum| exported_names.push(e.name.clone()));
        namespace.visit(|r: &Record| exported_names.push(r.name.clone()));
        namespace.visit(|f: &Function| exported_names.push(f.callable.name.clone()));
        namespace.visit(|i: &Interface| {
            exported_names.push(i.name.clone());
            exported_names.push(i.protocol.name.clone());
        });
        namespace.visit(|c: &CallbackInterface| exported_names.push(c.name.clone()));
        namespace.exported_names = exported_names;

        Ok(())
    })?;
    Ok(())
}
