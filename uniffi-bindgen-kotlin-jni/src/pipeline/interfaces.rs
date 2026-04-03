/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_class(int: general::Interface, context: &Context) -> Result<Class> {
    let methods = int.methods.map_node(context)?;
    let self_type = int.self_type.map_node(context)?;
    let interface_name = format!("{}Interface", int.name.to_upper_camel_case());
    let mut base_classes = vec![
        interface_name.clone(),
        "uniffi.Disposable".to_string(),
        "AutoCloseable".to_string(),
    ];
    if self_type.is_used_as_error {
        base_classes.push("kotlin.Exception".into());
    }

    Ok(Class {
        self_type,
        base_classes,
        constructors: int.constructors.map_node(context)?,
        methods,
        name: int.name.map_node(context)?,
        module_path: int.module_path,
        docstring: int.docstring.map_node(context)?,
        crate_name: context.current_crate_name()?.to_string(),
    })
}

pub fn map_interface(int: &general::Interface, context: &Context) -> Result<Interface> {
    Ok(Interface {
        name: format!("{}Interface", int.name.to_upper_camel_case()),
        methods: int.methods.clone().map_node(context)?,
        docstring: int.docstring.clone(),
    })
}
