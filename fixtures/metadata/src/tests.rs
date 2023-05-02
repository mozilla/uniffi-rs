/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// This entire crate is just a set of tests for metadata handling.  We use a separate crate
/// for testing because the metadata handling is split up between several crates, and no crate
/// owns all the functionality.
use crate::UniFfiTag;
use uniffi_meta::*;

mod person {
    #[derive(uniffi::Record, Debug)]
    pub struct Person {
        name: String,
        age: u16,
    }
}

mod weapon {
    #[derive(uniffi::Enum, Debug)]
    pub enum Weapon {
        Rock,
        Paper,
        Scissors,
    }
}

mod state {
    use super::Person;

    #[derive(uniffi::Enum, Debug)]
    pub enum State {
        Uninitialized,
        Initialized { data: String },
        Complete { result: Person },
    }
}

mod error {
    use super::Weapon;
    use std::fmt;

    #[derive(uniffi::Error)]
    #[uniffi(flat_error)]
    #[allow(dead_code)]
    pub enum FlatError {
        Overflow(String), // UniFFI should ignore this field, since `flat_error` was specified
        DivideByZero,
    }

    #[derive(uniffi::Error)]
    pub enum ComplexError {
        NotFound,
        PermissionDenied { reason: String },
        InvalidWeapon { weapon: Weapon },
    }

    impl fmt::Display for FlatError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Overflow(s) => write!(f, "FlatError::Overflow({s})"),
                Self::DivideByZero => write!(f, "FlatError::DivideByZero"),
            }
        }
    }

    impl fmt::Display for ComplexError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::NotFound => write!(f, "ComplexError::NotFound()"),
                Self::PermissionDenied { reason } => {
                    write!(f, "ComplexError::PermissionDenied({reason})")
                }
                Self::InvalidWeapon { weapon } => {
                    write!(f, "ComplexError::InvalidWeapon({weapon:?})")
                }
            }
        }
    }
}

mod calc {
    #[derive(uniffi::Object)]
    pub struct Calculator {}
}

pub use calc::Calculator;
pub use error::{ComplexError, FlatError};
pub use person::Person;
pub use state::State;
pub use weapon::Weapon;

mod test_type_ids {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uniffi_core::FfiConverter;

    fn check_type_id<T: FfiConverter<UniFfiTag>>(correct_type: Type) {
        let buf = &mut T::TYPE_ID_META.as_ref();
        assert_eq!(
            uniffi_meta::read_metadata_type(buf).unwrap(),
            correct_type,
            "Expected: {correct_type:?} data: {:?}",
            T::TYPE_ID_META.as_ref()
        );
    }

    #[test]
    fn simple_types() {
        check_type_id::<u8>(Type::U8);
        check_type_id::<u16>(Type::U16);
        check_type_id::<u32>(Type::U32);
        check_type_id::<u64>(Type::U64);
        check_type_id::<i8>(Type::I8);
        check_type_id::<i16>(Type::I16);
        check_type_id::<i32>(Type::I32);
        check_type_id::<i64>(Type::I64);
        check_type_id::<f32>(Type::F32);
        check_type_id::<f64>(Type::F64);
        check_type_id::<bool>(Type::Bool);
        check_type_id::<String>(Type::String);
    }

    #[test]
    fn test_user_types() {
        check_type_id::<Person>(Type::Record {
            name: "Person".into(),
        });
        check_type_id::<Weapon>(Type::Enum {
            name: "Weapon".into(),
        });
        check_type_id::<Arc<Calculator>>(Type::ArcObject {
            object_name: "Calculator".into(),
            is_trait: false,
        });
    }

    #[test]
    fn test_generics() {
        check_type_id::<Option<u8>>(Type::Option {
            inner_type: Box::new(Type::U8),
        });
        check_type_id::<Vec<u8>>(Type::Vec {
            inner_type: Box::new(Type::U8),
        });
        check_type_id::<HashMap<String, u8>>(Type::HashMap {
            key_type: Box::new(Type::String),
            value_type: Box::new(Type::U8),
        });
    }
}

fn check_metadata(encoded: &[u8], correct_metadata: impl Into<Metadata>) {
    assert_eq!(
        uniffi_meta::read_metadata(encoded).unwrap(),
        correct_metadata.into()
    )
}

mod test_metadata {
    use super::*;

    #[test]
    fn test_record() {
        check_metadata(
            &person::UNIFFI_META_UNIFFI_FIXTURE_METADATA_RECORD_PERSON,
            RecordMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "Person".into(),
                fields: vec![
                    FieldMetadata {
                        name: "name".into(),
                        ty: Type::String,
                    },
                    FieldMetadata {
                        name: "age".into(),
                        ty: Type::U16,
                    },
                ],
            },
        );
    }

    #[test]
    fn test_simple_enum() {
        check_metadata(
            &weapon::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_WEAPON,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "Weapon".into(),
                variants: vec![
                    VariantMetadata {
                        name: "Rock".into(),
                        fields: vec![],
                    },
                    VariantMetadata {
                        name: "Paper".into(),
                        fields: vec![],
                    },
                    VariantMetadata {
                        name: "Scissors".into(),
                        fields: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn test_complex_enum() {
        check_metadata(
            &state::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_STATE,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "State".into(),
                variants: vec![
                    VariantMetadata {
                        name: "Uninitialized".into(),
                        fields: vec![],
                    },
                    VariantMetadata {
                        name: "Initialized".into(),
                        fields: vec![FieldMetadata {
                            name: "data".into(),
                            ty: Type::String,
                        }],
                    },
                    VariantMetadata {
                        name: "Complete".into(),
                        fields: vec![FieldMetadata {
                            name: "result".into(),
                            ty: Type::Record {
                                name: "Person".into(),
                            },
                        }],
                    },
                ],
            },
        );
    }

    #[test]
    fn test_simple_error() {
        check_metadata(
            &error::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ERROR_FLATERROR,
            ErrorMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "FlatError".into(),
                flat: true,
                variants: vec![
                    VariantMetadata {
                        name: "Overflow".into(),
                        fields: vec![],
                    },
                    VariantMetadata {
                        name: "DivideByZero".into(),
                        fields: vec![],
                    },
                ],
            },
        );
    }

    #[test]
    fn test_complex_error() {
        check_metadata(
            &error::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ERROR_COMPLEXERROR,
            ErrorMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "ComplexError".into(),
                flat: false,
                variants: vec![
                    VariantMetadata {
                        name: "NotFound".into(),
                        fields: vec![],
                    },
                    VariantMetadata {
                        name: "PermissionDenied".into(),
                        fields: vec![FieldMetadata {
                            name: "reason".into(),
                            ty: Type::String,
                        }],
                    },
                    VariantMetadata {
                        name: "InvalidWeapon".into(),
                        fields: vec![FieldMetadata {
                            name: "weapon".into(),
                            ty: Type::Enum {
                                name: "Weapon".into(),
                            },
                        }],
                    },
                ],
            },
        );
    }

    #[test]
    fn test_interface() {
        check_metadata(
            &calc::UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_CALCULATOR,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "Calculator".into(),
                is_trait: false,
            },
        );
    }
}

mod test_function_metadata {
    use super::*;
    use std::sync::Arc;

    #[uniffi::export]
    #[allow(unused)]
    pub fn test_func(person: Person, weapon: Weapon) -> String {
        unimplemented!()
    }

    #[uniffi::export]
    pub fn test_func_no_return() {
        unimplemented!()
    }

    #[uniffi::export]
    pub fn test_func_that_throws() -> Result<State, FlatError> {
        unimplemented!()
    }

    #[uniffi::export]
    pub fn test_func_no_return_that_throws() -> Result<(), FlatError> {
        unimplemented!()
    }

    #[uniffi::export]
    #[allow(unused)]
    pub async fn test_async_func(person: Person, weapon: Weapon) -> String {
        unimplemented!()
    }

    #[uniffi::export]
    pub async fn test_async_func_that_throws() -> Result<State, FlatError> {
        unimplemented!()
    }

    #[uniffi::export]
    pub trait CalculatorDisplay: Send + Sync {
        fn display_result(&self, val: String);
    }

    #[uniffi::export]
    impl Calculator {
        #[allow(unused)]
        pub fn add(&self, a: u8, b: u8) -> u8 {
            unimplemented!()
        }

        #[allow(unused)]
        pub async fn async_sub(&self, a: u8, b: u8) -> u8 {
            unimplemented!()
        }

        #[allow(unused)]
        pub fn get_display(&self) -> Arc<dyn CalculatorDisplay> {
            unimplemented!()
        }
    }

    #[test]
    fn test_function() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_func".into(),
                is_async: false,
                inputs: vec![
                    FnParamMetadata {
                        name: "person".into(),
                        ty: Type::Record {
                            name: "Person".into(),
                        },
                    },
                    FnParamMetadata {
                        name: "weapon".into(),
                        ty: Type::Enum {
                            name: "Weapon".into(),
                        },
                    },
                ],
                return_type: Some(Type::String),
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC.checksum(),
            },
        );
    }

    #[test]
    fn test_function_no_return() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_func_no_return".into(),
                is_async: false,
                inputs: vec![],
                return_type: None,
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN
                    .checksum(),
            },
        );
    }

    #[test]
    fn test_function_that_throws() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_func_that_throws".into(),
                is_async: false,
                inputs: vec![],
                return_type: Some(Type::Enum {
                    name: "State".into(),
                }),
                throws: Some(Type::Error {
                    name: "FlatError".into(),
                }),
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_THAT_THROWS
                    .checksum(),
            },
        );
    }

    #[test]
    fn test_function_that_throws_no_return() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_func_no_return_that_throws".into(),
                is_async: false,
                inputs: vec![],
                return_type: None,
                throws: Some(Type::Error {
                    name: "FlatError".into(),
                }),
                checksum:
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN_THAT_THROWS
                        .checksum(),
            },
        );
    }

    #[test]
    fn test_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_ADD,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                self_name: "Calculator".into(),
                self_is_trait: false,
                name: "add".into(),
                is_async: false,
                inputs: vec![
                    FnParamMetadata {
                        name: "a".into(),
                        ty: Type::U8,
                    },
                    FnParamMetadata {
                        name: "b".into(),
                        ty: Type::U8,
                    },
                ],
                return_type: Some(Type::U8),
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_ADD
                    .checksum(),
            },
        );
    }

    #[test]
    fn test_async_function() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_async_func".into(),
                is_async: true,
                inputs: vec![
                    FnParamMetadata {
                        name: "person".into(),
                        ty: Type::Record {
                            name: "Person".into(),
                        },
                    },
                    FnParamMetadata {
                        name: "weapon".into(),
                        ty: Type::Enum {
                            name: "Weapon".into(),
                        },
                    },
                ],
                return_type: Some(Type::String),
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC.checksum(),
            },
        );
    }

    #[test]
    fn test_async_function_that_throws() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                name: "test_async_func_that_throws".into(),
                is_async: true,
                inputs: vec![],
                return_type: Some(Type::Enum {
                    name: "State".into(),
                }),
                throws: Some(Type::Error {
                    name: "FlatError".into(),
                }),
                checksum:
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC_THAT_THROWS
                        .checksum(),
            },
        );
    }

    #[test]
    fn test_async_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_ASYNC_SUB,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                self_name: "Calculator".into(),
                self_is_trait: false,
                name: "async_sub".into(),
                is_async: true,
                inputs: vec![
                    FnParamMetadata {
                        name: "a".into(),
                        ty: Type::U8,
                    },
                    FnParamMetadata {
                        name: "b".into(),
                        ty: Type::U8,
                    },
                ],
                return_type: Some(Type::U8),
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_ASYNC_SUB
                    .checksum(),
            },
        );
    }

    #[test]
    fn test_trait_result() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_GET_DISPLAY,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                self_name: "Calculator".into(),
                self_is_trait: false,
                name: "get_display".into(),
                is_async: false,
                inputs: vec![],
                return_type: Some(Type::ArcObject {
                    object_name: "CalculatorDisplay".into(),
                    is_trait: true,
                }),
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATOR_GET_DISPLAY
                    .checksum(),
            },
        );
    }

    #[test]
    fn test_trait_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORDISPLAY_DISPLAY_RESULT,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata".into(),
                self_name: "CalculatorDisplay".into(),
                self_is_trait: true,
                name: "display_result".into(),
                is_async: false,
                inputs: vec![
                    FnParamMetadata {
                        name: "val".into(),
                        ty: Type::String,
                    },
                ],
                return_type: None,
                throws: None,
                checksum: UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORDISPLAY_DISPLAY_RESULT
                    .checksum(),
            },
        );
    }
}
