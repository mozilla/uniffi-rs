/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use thiserror::Error as ThisError;
use uniffi::{Error, Record};

#[derive(Clone, Debug, Record)]
pub struct ErrorPayload {
    pub is_important: bool,
    pub index: u64,
    pub message: String,
}
impl Default for ErrorPayload {
    fn default() -> Self {
        Self {
            is_important: true,
            index: 42,
            message: "Very important error payload that greatly helps with debugging".to_owned(),
        }
    }
}
impl std::fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "is_important: {}, index: {}, message: {}",
            self.is_important, self.index, self.message
        )
    }
}

#[repr(u32)]
#[derive(Clone, Debug, ThisError, Error)]
pub enum LargeError {
    #[error("Important debug description of what went wrong.")]
    Case1 = 10001,

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case2 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case3 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case4 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case5 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case6 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case7 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case8 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case9 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case10 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case11 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case12 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case13 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case14 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case15 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case16 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case17 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case18 { arg1: String },

    #[error("Important debug description of what went wrong, arg: '{arg1}'")]
    Case19 { arg1: String },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case20 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case21 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case22 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case23 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case24 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case25 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case26 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case27 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case28 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}'")]
    Case29 { arg1: u64, arg2: u32 },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case30 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case31 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case32 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case33 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case34 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case35 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case36 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case37 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case38 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}'")]
    Case39 { arg1: ErrorPayload },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case40 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case41 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case42 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case43 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case44 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case45 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case46 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case47 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case48 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case49 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case50 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case51 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case52 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case53 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case54 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case55 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case56 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case57 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case58 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case59 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case60 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case61 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case62 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case63 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case64 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case65 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case66 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case67 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case68 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case69 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case70 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case71 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case72 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case73 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case74 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case75 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case76 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case77 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case78 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case79 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case80 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case81 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case82 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case83 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case84 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case85 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case86 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case87 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case88 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case89 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case90 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case91 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case92 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case93 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case94 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case95 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case96 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case97 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case98 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case99 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },

    #[error("Important debug description of what went wrong, arg1: '{arg1}', arg2: '{arg2}', arg3: '{arg3}', arg4: '{arg4}'")]
    Case100 {
        arg1: ErrorPayload,
        arg2: ErrorPayload,
        arg3: ErrorPayload,
        arg4: ErrorPayload,
    },
}

impl LargeError {
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }
}

#[derive(Clone, Debug, Record)]
pub struct ErrorWithContext {
    pub error: LargeError,
    pub context: String,
}

#[uniffi::export]
pub fn error_message_from_error(error: &LargeError) -> String {
    format!("{}", error)
}

#[uniffi::export]
pub fn error_code_from_error(error: &LargeError) -> u32 {
    error.discriminant()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn discriminant() {
        assert_eq!(
            error_code_from_error(&LargeError::Case2 {
                arg1: "foo".to_owned()
            }),
            10002
        );

        assert_eq!(
            error_code_from_error(&LargeError::Case100 {
                arg1: ErrorPayload::default(),
                arg2: ErrorPayload::default(),
                arg3: ErrorPayload::default(),
                arg4: ErrorPayload::default()
            }),
            10100
        );
    }

    #[test]
    fn message() {
        assert_eq!(
            error_message_from_error(&LargeError::Case1),
            "Important debug description of what went wrong.".to_owned()
        );

        assert_eq!(
            error_message_from_error(&LargeError::Case2 {
                arg1: "foobar".to_owned()
            }),
            "Important debug description of what went wrong, arg: 'foobar'".to_owned()
        );
    }
}

uniffi::include_scaffolding!("large_error");
