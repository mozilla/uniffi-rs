/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeType};
use crate::codegen::{DurationTypeHandler, NewCodeType, TimestampTypeHandler};
use crate::interface::ComponentInterface;
use askama::Template;

// KotlinCodeType functions shared by TimestampTypeHandler and DurationTypeHandler
macro_rules! shared_funcs(() => {
    fn lift(&self, nm: &str) -> String {
        format!("lift{}({})", self.canonical_name(), nm)
    }

    fn read(&self, nm: &str) -> String {
        format!("read{}({})", self.canonical_name(), nm)
    }

    fn lower(&self, nm: &str) -> String {
        format!("lower{}({})", self.canonical_name(), names::var_name(nm))
    }

    fn write(&self, nm: &str, target: &str) -> String {
        format!("write{}({}, {})", self.canonical_name(), names::var_name(nm), target)
    }
});

impl KotlinCodeType for TimestampTypeHandler {
    fn nm(&self) -> String {
        "java.time.Instant".into()
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(TimestampTemplate)
    }

    shared_funcs!();
}

impl KotlinCodeType for DurationTypeHandler {
    fn nm(&self) -> String {
        "java.time.Duration".into()
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(DurationTemplate)
    }

    shared_funcs!();
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Timestamp.kt")]
struct TimestampTemplate;

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Duration.kt")]
struct DurationTemplate;
