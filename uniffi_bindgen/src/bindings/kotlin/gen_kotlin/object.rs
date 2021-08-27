/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use super::Config;
use crate::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{ComponentInterface, Object};
use askama::Template;

// Filters is used by ObjectTemplate.kt, which looks for the filters module here.
use super::filters;
pub struct ObjectCodeType {
    id: String,
}

impl ObjectCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for ObjectCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("Object{}", self.type_label(oracle))
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lower()", oracle.var_name(nm))
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!("{}.write({})", oracle.var_name(nm), target)
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read({})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} class is found in ObjectTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

// Decorator data for an object
//
// This combines info from the UDL and config files
struct DecoratorInfo {
    decorated_class: String,
    decorator_class: String,
    import: Option<String>,
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectTemplate.kt")]
pub struct KotlinObject {
    inner: Object,
    decorator_info: Option<DecoratorInfo>,
}

impl KotlinObject {
    pub fn new(inner: Object, _ci: &ComponentInterface, config: &Config) -> Self {
        let decorator_info = match (
            inner.decorated_class(),
            config
                .decorators
                .as_ref()
                .and_then(|map| map.get(inner.name())),
        ) {
            (Some(decorated_class), Some(config)) => Some(DecoratorInfo {
                decorated_class,
                decorator_class: config.class_name.clone(),
                import: config.import.clone(),
            }),
            _ => None,
        };

        Self {
            inner,
            decorator_info,
        }
    }

    pub fn inner(&self) -> &Object {
        &self.inner
    }
}

impl CodeDeclaration for KotlinObject {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        let mut imports = vec![
            "java.util.concurrent.atomic.AtomicLong",
            "java.util.concurrent.atomic.AtomicBoolean",
        ];
        if let Some(DecoratorInfo {
            import: Some(ref import),
            ..
        }) = self.decorator_info
        {
            imports.push(import);
        }
        Some(imports.into_iter().map(|s| s.into()).collect())
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectRuntime.kt")]
pub struct KotlinObjectRuntime {
    is_needed: bool,
}

impl KotlinObjectRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_object_definitions().is_empty(),
        }
    }
}

impl CodeDeclaration for KotlinObjectRuntime {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        if self.is_needed {
            Some(self.render().unwrap())
        } else {
            None
        }
    }
}
