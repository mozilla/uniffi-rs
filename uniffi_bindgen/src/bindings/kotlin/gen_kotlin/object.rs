/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::backend::{CodeBuilder, CodeOracle, CodeType, Literal};
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

    fn build_code(
        &self,
        _oracle: &dyn CodeOracle,
        builder: &mut CodeBuilder,
        ci: &ComponentInterface,
    ) {
        builder
            .add_imports(vec![
                "import java.util.concurrent.atomic.AtomicLong".to_owned(),
                "import java.util.concurrent.atomic.AtomicBoolean".to_owned(),
            ])
            .add_code_block(KotlinObject::new(
                ci.get_object_definition(&self.id).unwrap().clone(),
            ))
            .add_code_block(KotlinObjectRuntime);
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectTemplate.kt")]
pub struct KotlinObject {
    inner: Object,
}

impl KotlinObject {
    pub fn new(inner: Object) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &Object {
        &self.inner
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectRuntime.kt")]
pub struct KotlinObjectRuntime;
