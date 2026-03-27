/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use camino::Utf8PathBuf;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream},
};
use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::files::{with_files, FileId};

#[derive(Clone)]
pub struct Error {
    /// Error type
    pub kind: ErrorKind,
    /// File/span where the error happened
    location: Option<(FileId, Span)>,
    /// Context when the error happened
    ///
    /// This is mainly used to handle type aliases,
    /// see `expect/error_through_type_alias.txt` for an example.
    context: Vec<ContextItem>,
}

impl std::error::Error for Error {}

#[derive(Clone)]
struct ContextItem {
    source: FileId,
    span: Span,
    message: String,
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ErrorKind {
    #[error("Invalid type")]
    InvalidType,
    #[error("Invalid default")]
    InvalidDefault,
    #[error("Trait types must be prefixed with `dyn`")]
    TraitWithoutDyn,
    #[error("Expected trait")]
    ExpectedTrait,
    #[error("Invalid builtin type")]
    InvalidCustomTypeBuiltin,
    #[error("Invalid dyn trait")]
    InvalidDynTrait,
    #[error("Invalid generic argument")]
    InvalidGenericArg,
    #[error("Too many generic arguments")]
    TooManyGenericArgs,
    #[error("Missing generic argument")]
    MissingGenericArg,
    #[error("{expected} generic args expected")]
    InvalidGenericArgLength { expected: usize },
    #[error("Cycle detected")]
    CycleDetected,
    #[error("Unknown item")]
    NotFound,
    #[error("`super` invalid")]
    SuperInvalid,
    #[error("`crate` invalid")]
    CrateInvalid,
    #[error("`self` invalid")]
    SelfInvalid,
    #[error("Name conflict")]
    NameConflict,
    #[error("Invalid impl type")]
    InvalidImplType,
    #[error("Missing self type")]
    MissingSelfType,
    #[error("Invalid self type")]
    InvalidSelfType,
    #[error("Invalid return type")]
    InvalidReturnType,
    #[error("Invalid argument name")]
    InvalidArgName,
    #[error("Invalid argument type")]
    InvalidArgType,
    #[error("Enum and Error derives are mutually exclusive")]
    MultipleEnumDerives,
    #[error("Invalid attribute")]
    InvalidAttr,
    #[error("Invalid repr type")]
    InvalidRepr,
    #[error("Invalid discriminant")]
    InvalidDiscr,
    #[error("uniffi_parse_rs internal error: {0}")]
    InternalError(String),
    #[error("Invalid file path: {0}")]
    InvalidPath(Utf8PathBuf),
    #[error("Error reading {0} {1}")]
    ReadError(Utf8PathBuf, String),
    #[error("Parsing error: {0}")]
    SynError(String),
}

impl Error {
    pub fn new(source: FileId, span: Span, kind: ErrorKind) -> Self {
        Self {
            kind,
            location: Some((source, span)),
            context: vec![],
        }
    }

    pub fn context(mut self, source: FileId, span: Span, message: impl Into<String>) -> Self {
        self.context.push(ContextItem {
            source,
            span,
            message: message.into(),
        });
        self
    }

    pub fn new_syn(source: FileId, e: syn::Error) -> Self {
        Self::new(source, e.span(), ErrorKind::SynError(e.to_string()))
    }

    pub fn new_without_location(kind: ErrorKind) -> Self {
        Self {
            kind,
            location: None,
            context: vec![],
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new_without_location(ErrorKind::InternalError(message.into()))
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self.kind, ErrorKind::NotFound)
    }

    pub fn is_cycle_detected(&self) -> bool {
        matches!(self.kind, ErrorKind::CycleDetected)
    }

    fn build_diagnostic(&self) -> Diagnostic<usize> {
        let mut diagnostic = Diagnostic::error()
            .with_message(&self.kind)
            .with_notes(self.notes());
        if let Some((file_id, span)) = &self.location {
            diagnostic = diagnostic
                .with_label(Label::primary(file_id.0, span.byte_range()).with_message(&self.kind));
        }

        diagnostic.with_labels(
            self.context
                .iter()
                .rev()
                .map(|item| {
                    Label::secondary(item.source.0, item.span.byte_range())
                        .with_message(&item.message)
                })
                .collect(),
        )
    }

    pub fn print_to_console(&self) {
        with_files(|files| {
            let diagnostic = self.build_diagnostic();

            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = codespan_reporting::term::Config::default();

            term::emit_to_write_style(&mut writer.lock(), &config, files, &diagnostic)
                .expect("Error writing errors to terminal");
        })
    }

    pub fn notes(&self) -> Vec<String> {
        match self.kind {
            ErrorKind::NotFound => {
                vec![
                    "Do you need to add a UniFFI derive or uniffi::export?".into(),
                    "Does the item come from a non-UniFFI crate?".into(),
                ]
            }
            _ => vec![],
        }
    }
}

pub trait ErrorContext {
    fn context(self, source: FileId, elt: impl Spanned, message: impl Into<String>) -> Self;
}

impl<T> ErrorContext for Result<T, Error> {
    fn context(self, source: FileId, elt: impl Spanned, message: impl Into<String>) -> Self {
        self.map_err(|mut e| {
            e.context.push(ContextItem {
                source,
                span: elt.span(),
                message: message.into(),
            });
            e
        })
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        with_files(|files| {
            let diagnostic = self.build_diagnostic();
            let config = codespan_reporting::term::Config::default();
            write!(
                f,
                "{}",
                term::emit_into_string(&config, files, &diagnostic)
                    .expect("Error writing errors to terminal")
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Ir;

    #[test]
    fn test_invalid_type() {
        let ir = Ir::new_for_test(&["invalid_type"]);
        let expected = expect_test::expect_file!["./expect/invalid_type.txt"];
        expected.assert_eq(&format!("{:#?}", ir.into_metadata_group_map().unwrap_err()));
    }

    #[test]
    fn test_invalid_result() {
        let ir = Ir::new_for_test(&["invalid_result"]);
        let expected = expect_test::expect_file!["./expect/invalid_result.txt"];
        expected.assert_eq(&format!("{:#?}", ir.into_metadata_group_map().unwrap_err()));
    }

    #[test]
    fn test_error_through_type_alias() {
        let ir = Ir::new_for_test(&["error_through_type_alias"]);
        let expected = expect_test::expect_file!["./expect/error_through_type_alias.txt"];
        expected.assert_eq(&format!("{:#?}", ir.into_metadata_group_map().unwrap_err()));
    }
}
