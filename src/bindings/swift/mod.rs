/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod gen_swift;
pub use gen_swift::{Config, SwiftWrapper};

use super::super::interface::ComponentInterface;

/// Generate Swift bindings for the given ComponentInterface, as a string.
pub fn generate_python_bindings(ci: &ComponentInterface) -> Result<String> {
    let config = Config::from(&ci);
    use askama::Template;
    SwiftWrapper::new(config, &ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render Swift bindings"))
}

/// ...
pub fn write_swift_bindings(ci: &ComponentInterface, out_dir: &str) -> Result<()> {
    // ...
}
