/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{CodeBuilder, KotlinCodeType};
use crate::codegen::{MapTypeHandler, OptionalTypeHandler, SequenceTypeHandler};
use crate::interface::{ComponentInterface, Literal, Type};
use askama::Template;

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
            ffi_converter_name: self.ffi_converter_name(),
            inner: self.inner.clone(),
        })
    }
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
            ffi_converter_name: self.ffi_converter_name(),
            inner: self.inner.clone(),
        })
    }
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
            ffi_converter_name: self.ffi_converter_name(),
            inner: self.inner.clone(),
        })
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Optional.kt")]
struct OptionalTemplate {
    name: String,
    ffi_converter_name: String,
    inner: Type,
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Sequence.kt")]
struct SequenceTemplate {
    name: String,
    ffi_converter_name: String,
    inner: Type,
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Map.kt")]
struct MapTemplate {
    name: String,
    ffi_converter_name: String,
    inner: Type,
}
