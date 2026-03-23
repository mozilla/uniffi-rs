/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_namespace(input: general::Namespace, context: &Context) -> Result<Package> {
    let mut context = context.clone();
    context.update_from_namespace(&input);
    let config = Config::from_toml(input.config_toml)?;

    Ok(Package {
        name: match &config.package_name {
            Some(name) => name.clone(),
            None => format!("uniffi.{}", input.name),
        },
        crate_name: input.name,
        config,
        functions: input.functions.map_node(&context)?,
    })
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
