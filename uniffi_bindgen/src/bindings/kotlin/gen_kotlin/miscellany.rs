/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeDeclarations, KotlinCodeType, NewCodeType};
use crate::interface::types::{DurationTypeHandler, TimestampTypeHandler};
use crate::interface::ComponentInterface;
use crate::Result;
use askama::Template;

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

    fn declare_code(
        &self,
        declarations: &mut CodeDeclarations,
        _ci: &ComponentInterface,
    ) -> Result<()> {
        declarations.definitions.insert(TimestampTemplate)
    }

    shared_funcs!();
}

impl KotlinCodeType for DurationTypeHandler {
    fn nm(&self) -> String {
        "java.time.Duration".into()
    }

    fn declare_code(
        &self,
        declarations: &mut CodeDeclarations,
        _ci: &ComponentInterface,
    ) -> Result<()> {
        declarations.definitions.insert(DurationTemplate)
    }

    shared_funcs!();
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Timestamp.kt")]
struct TimestampTemplate;

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "Duration.kt")]
struct DurationTemplate;
