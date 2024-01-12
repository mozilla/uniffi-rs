/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

/// <docstring-enum>
#[derive(uniffi::Enum)]
enum EnumTest {
    /// <docstring-enum-variant>
    One,
    /// <docstring-enum-variant-2>
    Two,
}

/// <docstring-associated-enum>
#[derive(uniffi::Enum)]
pub enum AssociatedEnumTest {
    /// <docstring-associated-enum-variant>
    Test {
        /// <docstring-variant-field>
        code: i16,
    },
    /// <docstring-associated-enum-variant-2>
    Test2 { code: i16 },
}

/// <docstring-error>
#[derive(uniffi::Error, Debug, thiserror::Error)]
#[uniffi(flat_error)]
pub enum ErrorTest {
    /// <docstring-error-variant>
    #[error("Test")]
    One,
    /// <docstring-error-variant-2>
    #[error("Two")]
    Two,
}

/// <docstring-associated-error>
#[derive(uniffi::Error, Debug, thiserror::Error)]
pub enum AssociatedErrorTest {
    /// <docstring-associated-error-variant>
    #[error("Test")]
    Test { code: i16 },
    /// <docstring-associated-error-variant-2>
    #[error("Test2")]
    Test2 { code: i16 },
}

/// <docstring-object>
#[derive(uniffi::Object)]
pub struct ObjectTest {}

#[uniffi::export]
impl ObjectTest {
    /// <docstring-primary-constructor>
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(ObjectTest {})
    }

    /// <docstring-alternate-constructor>
    #[uniffi::constructor]
    pub fn new_alternate() -> Arc<Self> {
        Arc::new(ObjectTest {})
    }

    /// <docstring-method>
    pub fn test(&self) {}
}

/// <docstring-record>
#[derive(uniffi::Record)]
struct RecordTest {
    /// <docstring-record-field>
    test: i32,
}

/// <docstring-function>
#[uniffi::export]
pub fn test() -> Result<(), ErrorTest> {
    let _ = ErrorTest::One;
    let _ = ErrorTest::Two;
    Ok(())
}

/// <docstring-multiline-function>
/// <second-line>
#[uniffi::export]
pub fn test_multiline() {}

#[uniffi::export]
pub fn test_without_docstring() -> Result<(), AssociatedErrorTest> {
    Ok(())
}

/// <docstring-callback>
#[uniffi::export(callback_interface)]
pub trait CallbackTest {
    /// <docstring-callback-method>
    fn test(&self);
}

/// This is a very long multi line test docstring that exceeds 255 characters.
/// This is a very long multi line test docstring that exceeds 255 characters.
/// This is a very long multi line test docstring that exceeds 255 characters.
/// This is a very long multi line test docstring that exceeds 255 characters.
#[uniffi::export]
pub fn test_long_docstring() {}

uniffi::include_scaffolding!("docstring-proc-macro");
