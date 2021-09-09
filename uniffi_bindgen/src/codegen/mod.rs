/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Code generation tools
//!
//! This module provides code generation tools that are used by both the bindings and scaffolding
//! code.

#[macro_use]
mod dispatch;
mod codetype;
mod template;

pub use codetype::NewCodeType;
pub use dispatch::*;
pub use template::TemplateRenderSet;
