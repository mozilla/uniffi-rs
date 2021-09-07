/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeBuilder;
use crate::codegen::NewCodeType;
use crate::interface::{ComponentInterface, Literal};

type_dispatch! {
    /// Kotlin-specific type behavior
    pub(super) trait KotlinCodeType: NewCodeType {
        /// Name for this type in Kotlin code
        fn nm(&self) -> String;

        /// Add code needed for this type, then return the code builder back.
        fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
            code_builder
        }

        fn literal(&self, _literal: &Literal) -> String {
            unreachable!();
        }

        /// Name of the FFIConverter object for this type
        fn ffi_converter_name(&self) -> String {
            format!("FFIConverter{}", self.canonical_name())
        }

        /// lower function name for this type
        fn lower(&self) -> String {
            format!("{}.lower", self.ffi_converter_name())
        }

        /// write function name for this type
        fn write(&self) -> String {
            format!("{}.write", self.ffi_converter_name())
        }

        /// lift function name for this type
        fn lift(&self) -> String {
            format!("{}.lift", self.ffi_converter_name())
        }

        /// read function name for this type
        fn read(&self) -> String {
            format!("{}.read", self.ffi_converter_name())
        }
   }
}
