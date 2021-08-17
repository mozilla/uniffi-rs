/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


/// Context that the code generation is running in

use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct UniffiContext {
    pub crate_root: PathBuf,
}

impl UniffiContext {
    /// Get the path to a binding-specific file
    pub fn get_bindings_path(&self, binding_dir_name: impl AsRef<Path>, filename: impl AsRef<Path>) -> PathBuf {
        self.crate_root
            .join("uniffi")
            .join("bindings")
            .join(binding_dir_name)
            .join(filename)
    }
}
