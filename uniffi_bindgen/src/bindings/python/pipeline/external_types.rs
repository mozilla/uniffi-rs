/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use indexmap::IndexSet;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    let namespace_config = namespace.config.clone();
    let current_namespace_name = namespace.name.clone();
    let mut module_imports = IndexSet::new();
    namespace.visit_mut(|ty: &mut Type| {
        match ty {
            Type::Enum {
                namespace,
                external_package_name,
                ..
            }
            | Type::Record {
                namespace,
                external_package_name,
                ..
            }
            | Type::Interface {
                namespace,
                external_package_name,
                ..
            }
            | Type::CallbackInterface {
                namespace,
                external_package_name,
                ..
            }
            | Type::Custom {
                namespace,
                external_package_name,
                ..
            } => {
                if *namespace != current_namespace_name {
                    match namespace_config.external_packages.get(namespace) {
                        None => {
                            // No configuration, use the module name as a relative import
                            module_imports.insert(format!(".{namespace}"));
                            *external_package_name = Some(namespace.clone());
                        }
                        Some(value) if value.is_empty() => {
                            // Empty string, use the module name as an absolute import
                            module_imports.insert(namespace.clone());
                            *external_package_name = Some(namespace.clone());
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
    namespace.imports.extend(module_imports);
    Ok(())
}
