/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeType};
use crate::codegen::{MapTypeHandler, NewCodeType, OptionalTypeHandler, SequenceTypeHandler};
use crate::interface::{ComponentInterface, Literal, Type};
use askama::Template;

// KotlinCodeType functions shared by MapTemplate, SequenceTypeHandler, and OptionalTypeHandler
macro_rules! shared_funcs(() => {
    fn lower(&self, nm: &str) -> String {
        format!("lower{}({})", self.canonical_name(), names::var_name(nm))
    }

    fn write(&self, nm: &str, target: &str) -> String {
        format!(
            "write{}({}, {})",
            self.canonical_name(),
            names::var_name(nm),
            target
        )
    }

    fn lift(&self, nm: &str) -> String {
        format!("lift{}({})", self.canonical_name(), nm)
    }

    fn read(&self, nm: &str) -> String {
        format!("read{}({})", self.canonical_name(), nm)
    }
});

impl KotlinCodeType for MapTypeHandler<'_> {
    fn nm(&self) -> String {
        format!("Map<String, {}>", self.inner.nm())
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::EmptyMap => "mapOf".into(),
            _ => unreachable!(),
        }
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(MapTemplate {
            name: self.nm(),
            canonical_name: self.canonical_name(),
            inner: self.inner.clone(),
        })
    }

    shared_funcs!();
}

impl KotlinCodeType for SequenceTypeHandler<'_> {
    fn nm(&self) -> String {
        format!("List<{}>", self.inner.nm())
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::EmptySequence => "listOf()".into(),
            _ => unreachable!(),
        }
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(SequenceTemplate {
            name: self.nm(),
            canonical_name: self.canonical_name(),
            inner: self.inner.clone(),
        })
    }

    shared_funcs!();
}

impl KotlinCodeType for OptionalTypeHandler<'_> {
    fn nm(&self) -> String {
        format!("{}?", self.inner.nm())
    }

    fn literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::Null => "null".into(),
            _ => self.inner.literal(literal),
        }
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(OptionalTemplate {
            name: self.nm(),
            canonical_name: self.canonical_name(),
            inner: self.inner.clone(),
        })
    }

    shared_funcs!();
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Optional.kt")]
struct OptionalTemplate {
    name: String,
    canonical_name: String,
    inner: Type,
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Sequence.kt")]
struct SequenceTemplate {
    name: String,
    canonical_name: String,
    inner: Type,
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Map.kt")]
struct MapTemplate {
    name: String,
    canonical_name: String,
    inner: Type,
}
