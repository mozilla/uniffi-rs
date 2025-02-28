/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;

use crate::nanopass::{ir, passes};

ir! {
    extend passes::last;

    // struct Module {
    //     +config: Option<Config>,
    // }
    //
    // // Config options to customize the generated python.
    // +struct Config {
    //     cdylib_name: Option<String>,
    //     custom_types: IndexMap<String, CustomTypeConfig>,
    //     external_packages: IndexMap<String, String>,
    // }
    //
    // +pub struct CustomTypeConfig {
    //     imports: Option<Vec<String>>,
    //     into_custom: String,
    //     from_custom: String,
    // }
}

pub fn pass(root: &mut Root) -> Result<()> {
    Ok(())
}

//
//     fn add_module_config(module: &prev::Module) -> Result<Option<Config>> {
//         let Some(config_path) = module.config_path.as_ref() else {
//             return Ok(None)
//         };
//         let content = fs::read_to_string(config_path)
//             .with_context(|| format!("Error reading config {config_path}"))?;
//         let entire_config: deserialize::FullConfig = toml::from_str(&content)?;
//         let config: Option<Config> = entire_config.bindings
//             .and_then(|bindings| bindings.python)
//             .map(|config| Config {
//                 cdylib_name: config.cdylib_name,
//                 external_packages: config.external_packages,
//                 custom_types: config.custom_types
//                     .into_iter()
//                     .map(|(key, custom_type_config)| {
//                         let custom_type_config = CustomTypeConfig {
//                             imports: custom_type_config.imports,
//                             into_custom: custom_type_config
//                                 .into_custom
//                                 .replace("{}", "builtin_value"),
//                             from_custom: custom_type_config
//                                 .from_custom
//                                 .replace("{}", "value"),
//                         };
//                         (key, custom_type_config)
//                     })
//                     .collect()
//             });
//         Ok(config)
//     }
// }
//
// /// Put the deserialization code in a separate mod, we don't want the `Deserialize` derive to move
// /// to future passes.
// mod deserialize {
//     use indexmap::IndexMap;
//     use serde::Deserialize;
//
//     #[derive(Deserialize)]
//     pub struct FullConfig {
//         pub bindings: Option<BindingsConfig>,
//     }
//
//     #[derive(Deserialize)]
//     pub struct BindingsConfig {
//         pub python: Option<Config>,
//     }
//
//     // Config options to customize the generated python.
//     #[derive(Deserialize)]
//     pub struct Config {
//         pub cdylib_name: Option<String>,
//         #[serde(default)]
//         pub custom_types: IndexMap<String, CustomTypeConfig>,
//         #[serde(default)]
//         pub external_packages: IndexMap<String, String>,
//     }
//
//     #[derive(Deserialize)]
//     pub struct CustomTypeConfig {
//         pub imports: Option<Vec<String>>,
//         pub into_custom: String,
//         pub from_custom: String,
//     }
// }
