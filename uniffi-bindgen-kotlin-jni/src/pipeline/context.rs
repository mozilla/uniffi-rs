/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    pub current_crate_name: Option<String>,
}

impl Context {
    pub fn update_from_namespace(&mut self, namespace: &general::Namespace) {
        self.current_crate_name = Some(namespace.crate_name.clone());
    }

    pub fn crate_name(&self) -> Result<&str> {
        self.current_crate_name
            .as_deref()
            .ok_or_else(|| anyhow!("current_crate_name not set"))
    }
}
