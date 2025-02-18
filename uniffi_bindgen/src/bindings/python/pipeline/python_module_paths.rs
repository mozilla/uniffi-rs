/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Convert module names into Python module paths

use heck::ToSnakeCase;

use super::config as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    enum Type {
        Interface {
            -module_name,
            +module_path: String,
        },
        Record {
            -module_name,
            +module_path: String,
        },
        Enum {
            -module_name,
            +module_path: String,
        },
        CallbackInterface {
            -module_name,
            +module_path: String,
        },
        Custom {
            -module_name,
            +module_path: String,
        },
    }

    struct Context {
        #[nanopass(from(Module::config))]
        config: Option<prev::Config>,
    }

    fn add_type_interface_module_path(int: &prev::VariantTypeInterface, context: &Context) -> Result<String> {
        module_path(&int.module_name, context)
    }

    fn add_type_record_module_path(rec: &prev::VariantTypeRecord, context: &Context) -> Result<String> {
        module_path(&rec.module_name, context)
    }

    fn add_type_enum_module_path(en: &prev::VariantTypeEnum, context: &Context) -> Result<String> {
        module_path(&en.module_name, context)
    }

    fn add_type_callbackinterface_module_path(cbi: &prev::VariantTypeCallbackInterface, context: &Context) -> Result<String> {
        module_path(&cbi.module_name, context)
    }


    fn add_type_custom_module_path(custom: &prev::VariantTypeCustom, context: &Context) -> Result<String> {
        module_path(&custom.module_name, context)
    }
}

fn module_path(module_name: &str, context: &Context) -> Result<String> {
    let external_packages = context.config.as_ref().map(|c| &c.external_packages);
    let module_name = module_name.to_snake_case();
    Ok(
        match external_packages.and_then(|packages| packages.get(&module_name)) {
            // By default, assume a module in the same package as the generated bindings
            None => format!(".{module_name}"),
            // An empty string in the config means a top-level module
            Some(value) if value.is_empty() => module_name,
            // Otherwise, it's a module inside the package specified in the config
            Some(value) => format!("{value}.{module_name}"),
        },
    )
}
