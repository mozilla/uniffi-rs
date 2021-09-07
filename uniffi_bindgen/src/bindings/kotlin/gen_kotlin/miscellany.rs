/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{CodeBuilder, KotlinCodeType};
use crate::codegen::{DurationTypeHandler, TimestampTypeHandler};
use crate::interface::ComponentInterface;
use askama::Template;

impl KotlinCodeType for TimestampTypeHandler {
    fn nm(&self) -> String {
        "java.time.Instant".into()
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(TimestampTemplate {
            ffi_converter_name: self.ffi_converter_name(),
        })
    }
}

impl KotlinCodeType for DurationTypeHandler {
    fn nm(&self) -> String {
        "java.time.Duration".into()
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(DurationTemplate {
            ffi_converter_name: self.ffi_converter_name(),
        })
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Timestamp.kt")]
struct TimestampTemplate {
    ffi_converter_name: String,
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Duration.kt")]
struct DurationTemplate {
    ffi_converter_name: String,
}
