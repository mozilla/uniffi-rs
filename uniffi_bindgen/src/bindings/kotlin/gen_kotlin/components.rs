/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{filters, Config, CustomTypeConfig};
use crate::interface::*;
use askama::Template;
/// Renders types, functions, and other components from the `ComponentInterface`
///
/// One source of complexity here is that the code for a single component can be scattered across
/// the final bindings.  The two main examples are imports, which go at the top of the file, and
/// initialization code, which needs to be inside the lazy closure for `_UniFFILib`.  We use the
/// following system to handle this:
///
/// - The main code gets rendered in the `Components.kt` template.  That template iterates over all
///   types then forwards rendering to a template dedicated to the type variant.  It also iterates
///   over all functions and forwards rendering to `TopLevelFunctionTemplate.kt`
/// - That component templates can call the `add_import()` and `add_initialization_code` methods on
///   `Components` to arrange for code to be placed to other sections.
/// - The rendered code, plus the imports and initialization code gets combined into a
///   `CodeBlocks` struct.
/// - The final bindings is rendered by the `KotlinBindings` template in a new pass.  That
///   templates takes the `CodeBlocks` struct and for the code to be placed in the correct place.
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

/// Render code for ComponentInterface types and functions
pub(super) fn render(ci: &ComponentInterface, config: &Config) -> CodeBlocks {
    let template = Components::new(ci, config);
    let main = template.render().unwrap();

    CodeBlocks {
        main,
        imports: template.imports.into_inner(),
        initialization_code: template.initialization_code.into_inner(),
    }
}

pub(super) struct CodeBlocks {
    // Main declaration code block
    pub(super) main: String,
    // Import statements, sorted alphabetically by the `BTreeSet`.
    pub(super) imports: BTreeSet<String>,
    // Statements that should run on when we're initializing the library
    pub(super) initialization_code: Vec<String>,
}

/// Render the main code block for these bindings
//
// This template breaks the askama model a bit since it contains data that gets mutated during the
// render process.  We use a few tricks to handle this:
//
//   - Wrap the mutable data in a RefCell<>
//   - Mutator methods that logically return `()` return the empty string.  That allows us to use
//     an Askama expression to call them.
//   - We define a macro in `macros.kt` so that this all looks more natural when reading the code.
//     (`{% call add_import(...) %}` vs `{{ self.add_import(...) }}`)
#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "Components.kt")]
struct Components<'a> {
    ci: &'a ComponentInterface,
    config: &'a Config,
    imports: RefCell<BTreeSet<String>>,
    initialization_code: RefCell<Vec<String>>,
    included_templates: RefCell<HashSet<String>>,
}

impl<'a> Components<'a> {
    fn new(ci: &'a ComponentInterface, config: &'a Config) -> Self {
        Self {
            ci,
            config,
            imports: BTreeSet::new().into(),
            initialization_code: Vec::new().into(),
            included_templates: HashSet::new().into(),
        }
    }

    fn get_custom_type_config(&self, name: &str) -> Option<&CustomTypeConfig> {
        self.config.custom_types.get(name)
    }

    /// Add an import statement at the top of the bindings
    fn add_import(&self, statement: &str) -> &str {
        self.imports.borrow_mut().insert(statement.to_owned());
        ""
    }

    /// Add imports from Option<Vec<statements>>, like in the custom type config
    fn add_optional_imports(&self, optional_imports: &Option<Vec<String>>) -> &str {
        let mut imports = self.imports.borrow_mut();
        if let Some(import_statements) = optional_imports {
            for statement in import_statements {
                imports.insert(statement.clone());
            }
        }
        ""
    }

    /// Add initialization code to be run once the dynamic library is loaded
    fn add_initialization_code(&self, statement: &str) -> &str {
        self.initialization_code
            .borrow_mut()
            .push(statement.to_owned());
        ""
    }

    /// This can be used to implement an include_once style include.  Call this with the name of
    /// the template you want to include and only include the file if it returns true.  See
    /// `CallbackInterfaceTemplate.kt` for an example.
    fn include_once_check(&self, template_name: &str) -> bool {
        let mut included_templates = self.included_templates.borrow_mut();
        if included_templates.contains(template_name) {
            false
        } else {
            included_templates.insert(template_name.to_owned());
            true
        }
    }
}
