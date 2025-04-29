/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use indexmap::IndexSet;

pub fn pass(module: &mut Module) -> Result<()> {
    let module_config = module.config.clone();
    let current_module_name = module.name.clone();
    let mut module_imports = IndexSet::new();
    module.visit_mut(|ty: &mut Type| {
        match ty {
            Type::Enum {
                module_name,
                external_package_name,
                ..
            }
            | Type::Record {
                module_name,
                external_package_name,
                ..
            }
            | Type::Interface {
                module_name,
                external_package_name,
                ..
            }
            | Type::CallbackInterface {
                module_name,
                external_package_name,
                ..
            }
            | Type::Custom {
                module_name,
                external_package_name,
                ..
            } => {
                if *module_name != current_module_name {
                    match module_config.external_packages.get(module_name) {
                        None => {
                            // No configuration, use the module name as a relative import
                            module_imports.insert(format!(".{module_name}"));
                            *external_package_name = Some(module_name.clone());
                        }
                        Some(value) if value.is_empty() => {
                            // Empty string, use the module name as an absolute import
                            module_imports.insert(module_name.clone());
                            *external_package_name = Some(module_name.clone());
                        }
                        Some(package_name) => {
                            // Package name for configuration, use that name
                            module_imports.insert(package_name.clone());
                            *external_package_name = Some(package_name.clone());
                        }
                    };
                }
            }
            _ => (),
        };
    });
    module.imports.extend(module_imports);
    Ok(())
}
