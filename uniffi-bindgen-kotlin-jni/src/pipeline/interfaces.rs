/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_class(int: general::Interface, context: &Context) -> Result<Class> {
    let mut base_classes = vec![
        interface_name(&int),
        "uniffi.Disposable".to_string(),
        "AutoCloseable".to_string(),
    ];
    let callback_interface = int
        .imp
        .is_trait_interface()
        .then(|| callbacks::map_trait_interface(int.clone(), context))
        .transpose()?;

    let methods = int.methods.map_node(context)?;
    let self_type = int.self_type.map_node(context)?;
    if self_type.is_used_as_error {
        base_classes.push("kotlin.Exception".into());
    }

    Ok(Class {
        self_type,
        base_classes,
        package_name: context.current_package_name()?.to_string(),
        constructors: int.constructors.map_node(context)?,
        methods,
        name: int.name,
        orig_name: int.orig_name,
        module_path: int.module_path,
        docstring: int.docstring.map_node(context)?,
        crate_name: context.current_crate_name()?.to_string(),
        imp: int.imp,
        callback_interface,
    })
}

pub fn map_interface(int: &general::Interface, context: &Context) -> Result<Interface> {
    Ok(Interface {
        name: interface_name(int),
        methods: int.methods.clone().map_node(context)?,
        docstring: int.docstring.clone(),
    })
}

fn interface_name(int: &general::Interface) -> String {
    if int.imp.has_callback_interface() {
        int.name.to_upper_camel_case()
    } else {
        format!("{}Interface", int.name.to_upper_camel_case())
    }
}
