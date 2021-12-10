/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use std::collections::BTreeSet;

/// Container for generated code
///
/// When we generate bindings or scaffolding code, we is to iterate over types, functions, and
/// other components, and call their `build_code` method, passing them a `CodeBuilder` to add
/// code to.  Each language handles this slightly differently, but the general picture is the same.
///
/// Each of our fields stores some generated code, we use BTreeSet so that:
///   - If two components generate the same code, then we avoid duplicates in the final product.
///     For example, if we have 2 callback interface types, we don't want to include the callback
///     runtime template twice.
///   - The imports get sorted into a nice alphabetical order.
///
/// All code-builder methods use a builder-style API, which makes it easy to chain together calls
/// when adding different kinds of code.
#[derive(Default)]
pub struct CodeBuilder {
    // Blocks of code for the main code section.
    pub code_blocks: BTreeSet<String>,
    // Import statements.
    pub import_statements: BTreeSet<String>,
    // Lines of code to run once we load the shared library.
    pub initialization_code: BTreeSet<String>,
}

impl CodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_code_block(&mut self, template: impl Template) -> &mut Self {
        self.code_blocks.insert(template.render().unwrap());
        self
    }

    pub fn add_imports(&mut self, imports: Vec<String>) -> &mut Self {
        for import in imports {
            self.import_statements.insert(import);
        }
        self
    }

    pub fn add_initialization_code(&mut self, s: String) -> &mut Self {
        self.initialization_code.insert(s);
        self
    }
}
