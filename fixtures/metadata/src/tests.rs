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
    #[uniffi(name = "PersonRenamed")]
    pub struct Person {
        #[uniffi(default = "test", name = "name_renamed")]
        name: String,
        #[uniffi(default)]
        preferred_name: String,
        age: Box<u16>,
    }
}

mod weapon {
    #[derive(uniffi::Enum, Debug)]
    #[uniffi(name = "WeaponRenamed")]
    pub enum Weapon {
        Rock,
        Paper,
        #[uniffi(name = "ScissorsRenamed")]
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

    #[uniffi::export]
    impl State {
        #[uniffi::method(name = "state_method_renamed")]
        fn state_method(&self) {}
    }
}

mod enum_repr {
    #[derive(uniffi::Enum, Debug)]
    #[repr(u8)]
    pub enum ReprU8 {
        One = 1,
        Three = 3,
        Fifteen = 0x0F,
    }

    #[derive(uniffi::Enum, Debug)]
    pub enum NoRepr {
        One = 1,
    }
}

mod error {
    use super::Weapon;

    #[derive(Debug, thiserror::Error, uniffi::Error)]
    #[uniffi(flat_error)]
    #[allow(dead_code)]
    pub enum FlatError {
        #[error("Overflow")]
        Overflow(String), // UniFFI should ignore this field, since `flat_error` was specified
        #[error("DivideByZero")]
        DivideByZero,
    }

    #[derive(Debug, thiserror::Error, uniffi::Error)]
    pub enum ComplexError {
        #[error("NotFound")]
        NotFound,
        #[error("PermissionDenied")]
        PermissionDenied { reason: String },
        #[error("InvalidWeapon")]
        InvalidWeapon { weapon: Weapon },
    }
}

mod many_items {
    macro_rules! many_items {
        ($($name:ident),* $(,)?) => {
            #[derive(uniffi::Enum, Debug)]
            #[allow(non_camel_case_types)]
            pub enum ManyVariants {
                $($name),*
            }

            #[derive(Debug, thiserror::Error, uniffi::Error)]
            #[uniffi(flat_error)]
            #[allow(dead_code, non_camel_case_types)]
            pub enum ManyVariantsError {
                $(#[error("error")] $name),*
            }

            #[derive(uniffi::Record, Debug)]
            pub struct ManyFields {
                $(pub $name: u8),*
            }

            #[derive(uniffi::Enum, Debug)]
            pub enum ManyVariantFields {
                Variant { $($name: u8),* },
            }

            #[uniffi::export]
            #[allow(unused, clippy::too_many_arguments)]
            pub fn many_args($($name: u8),*) {}
        };
    }

    many_items!(
        x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19,
        x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31, x32, x33, x34, x35, x36, x37,
        x38, x39, x40, x41, x42, x43, x44, x45, x46, x47, x48, x49, x50, x51, x52, x53, x54, x55,
        x56, x57, x58, x59, x60, x61, x62, x63, x64, x65, x66, x67, x68, x69, x70, x71, x72, x73,
        x74, x75, x76, x77, x78, x79, x80, x81, x82, x83, x84, x85, x86, x87, x88, x89, x90, x91,
        x92, x93, x94, x95, x96, x97, x98, x99, x100, x101, x102, x103, x104, x105, x106, x107,
        x108, x109, x110, x111, x112, x113, x114, x115, x116, x117, x118, x119, x120, x121, x122,
        x123, x124, x125, x126, x127, x128, x129, x130, x131, x132, x133, x134, x135, x136, x137,
        x138, x139, x140, x141, x142, x143, x144, x145, x146, x147, x148, x149, x150, x151, x152,
        x153, x154, x155, x156, x157, x158, x159, x160, x161, x162, x163, x164, x165, x166, x167,
        x168, x169, x170, x171, x172, x173, x174, x175, x176, x177, x178, x179, x180, x181, x182,
        x183, x184, x185, x186, x187, x188, x189, x190, x191, x192, x193, x194, x195, x196, x197,
        x198, x199, x200, x201, x202, x203, x204, x205, x206, x207, x208, x209, x210, x211, x212,
        x213, x214, x215, x216, x217, x218, x219, x220, x221, x222, x223, x224, x225, x226, x227,
        x228, x229, x230, x231, x232, x233, x234, x235, x236, x237, x238, x239, x240, x241, x242,
        x243, x244, x245, x246, x247, x248, x249, x250, x251, x252, x253, x254, x255, x256, x257,
        x258, x259, x260, x261, x262, x263, x264, x265, x266, x267, x268, x269, x270, x271, x272,
        x273, x274, x275, x276, x277, x278, x279, x280, x281, x282, x283, x284, x285, x286, x287,
        x288, x289, x290, x291, x292, x293, x294, x295, x296, x297, x298, x299
    );
}

mod calc {
    #[derive(uniffi::Object)]
    #[uniffi(name = "CalculatorRenamed")]
    pub struct Calculator {}
}

mod uniffi_traits {
    #[derive(Debug, PartialEq, Eq, Ord, PartialOrd, uniffi::Object)]
    #[uniffi::export(Debug, Eq, Ord)]
    pub struct Special {}

    #[derive(Debug, PartialEq, Eq, Ord, PartialOrd, uniffi::Enum)]
    #[uniffi::export(Debug, Eq, Ord)]
    pub enum SpecialEnum {
        A,
    }

    #[derive(Debug, PartialEq, Eq, Ord, PartialOrd, uniffi::Record)]
    #[uniffi::export(Debug, Eq, Ord)]
    pub struct SpecialRecord {
        a: u8,
    }
}

#[uniffi::export(callback_interface)]
pub trait Logger {
    fn log(&self, message: String);
}

#[derive(uniffi::Object)]
pub struct RealLogger {}

#[uniffi::export]
impl Logger for RealLogger {
    fn log(&self, _message: String) {}
}

pub use calc::Calculator;
pub use error::FlatError;
pub use person::Person;
pub use state::State;

pub use weapon::Weapon;

mod test_type_ids {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uniffi_core::TypeId;

    fn check_type_id<T: TypeId<UniFfiTag>>(correct_type: Type) {
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
        check_type_id::<u8>(Type::UInt8);
        check_type_id::<u16>(Type::UInt16);
        check_type_id::<u32>(Type::UInt32);
        check_type_id::<u64>(Type::UInt64);
        check_type_id::<i8>(Type::Int8);
        check_type_id::<i16>(Type::Int16);
        check_type_id::<i32>(Type::Int32);
        check_type_id::<i64>(Type::Int64);
        check_type_id::<f32>(Type::Float32);
        check_type_id::<f64>(Type::Float64);
        check_type_id::<bool>(Type::Boolean);
        check_type_id::<String>(Type::String);
    }

    #[test]
    fn test_user_types() {
        check_type_id::<Person>(Type::Record {
            module_path: "uniffi_fixture_metadata::tests::person".into(),
            name: "PersonRenamed".into(),
        });
        check_type_id::<Weapon>(Type::Enum {
            module_path: "uniffi_fixture_metadata::tests::weapon".into(),
            name: "WeaponRenamed".into(),
        });
        check_type_id::<Arc<Calculator>>(Type::Object {
            module_path: "uniffi_fixture_metadata::tests::calc".into(),
            name: "CalculatorRenamed".into(),
            imp: ObjectImpl::Struct,
        });
    }

    #[test]
    fn test_generics() {
        check_type_id::<Option<u8>>(Type::Optional {
            inner_type: Box::new(Type::UInt8),
        });
        check_type_id::<Box<u8>>(Type::Box {
            inner_type: Box::new(Type::UInt8),
        });
        check_type_id::<Vec<u8>>(Type::Bytes);
        check_type_id::<Vec<u16>>(Type::Sequence {
            inner_type: Box::new(Type::UInt16),
        });
        check_type_id::<HashMap<String, u8>>(Type::Map {
            key_type: Box::new(Type::String),
            value_type: Box::new(Type::UInt8),
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
            &person::UNIFFI_META_UNIFFI_FIXTURE_METADATA_RECORD_PERSONRENAMED,
            RecordMetadata {
                module_path: "uniffi_fixture_metadata::tests::person".into(),
                name: "PersonRenamed".into(),
                orig_name: Some("Person".into()),
                remote: false,
                fields: vec![
                    FieldMetadata {
                        name: "name_renamed".into(),
                        orig_name: Some("name".into()),
                        ty: Type::String,
                        default: Some(DefaultValueMetadata::Literal(LiteralMetadata::String(
                            "test".to_owned(),
                        ))),
                        docstring: None,
                    },
                    FieldMetadata {
                        name: "preferred_name".into(),
                        orig_name: None,
                        ty: Type::String,
                        default: Some(DefaultValueMetadata::Default),
                        docstring: None,
                    },
                    FieldMetadata {
                        name: "age".into(),
                        orig_name: None,
                        ty: Type::Box {
                            inner_type: Box::new(Type::UInt16),
                        },
                        default: None,
                        docstring: None,
                    },
                ],
                docstring: None,
            },
        );
    }

    #[test]
    fn test_simple_enum() {
        check_metadata(
            &weapon::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_WEAPONRENAMED,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::weapon".into(),
                name: "WeaponRenamed".into(),
                orig_name: Some("Weapon".into()),
                shape: EnumShape::Enum,
                remote: false,
                discr_type: None,
                variants: vec![
                    VariantMetadata {
                        name: "Rock".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "Paper".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "ScissorsRenamed".into(),
                        orig_name: Some("Scissors".into()),
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                ],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_complex_enum() {
        check_metadata(
            &state::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_STATE,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::state".into(),
                name: "State".into(),
                orig_name: None,
                shape: EnumShape::Enum,
                remote: false,
                discr_type: None,
                variants: vec![
                    VariantMetadata {
                        name: "Uninitialized".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "Initialized".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![FieldMetadata {
                            name: "data".into(),
                            orig_name: None,
                            ty: Type::String,
                            default: None,
                            docstring: None,
                        }],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "Complete".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![FieldMetadata {
                            name: "result".into(),
                            orig_name: None,
                            ty: Type::Record {
                                module_path: "uniffi_fixture_metadata::tests::person".into(),
                                name: "PersonRenamed".into(),
                            },
                            default: None,
                            docstring: None,
                        }],
                        docstring: None,
                    },
                ],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_repr_enum() {
        check_metadata(
            &enum_repr::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_REPRU8,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::enum_repr".into(),
                name: "ReprU8".into(),
                orig_name: None,
                shape: EnumShape::Enum,
                remote: false,
                discr_type: Some(Type::UInt8),
                variants: vec![
                    VariantMetadata {
                        name: "One".into(),
                        orig_name: None,
                        discr: Some(LiteralMetadata::new_uint(1)),
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "Three".into(),
                        orig_name: None,
                        discr: Some(LiteralMetadata::new_uint(3)),
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "Fifteen".into(),
                        orig_name: None,
                        discr: Some(LiteralMetadata::new_uint(15)),
                        fields: vec![],
                        docstring: None,
                    },
                ],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_no_repr_enum() {
        check_metadata(
            &enum_repr::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_NOREPR,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::enum_repr".into(),
                name: "NoRepr".into(),
                orig_name: None,
                shape: EnumShape::Enum,
                remote: false,
                discr_type: None,
                variants: vec![VariantMetadata {
                    name: "One".into(),
                    orig_name: None,
                    discr: Some(LiteralMetadata::new_uint(1)),
                    fields: vec![],
                    docstring: None,
                }],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    fn many_variants_metadata() -> Vec<VariantMetadata> {
        (0..300)
            .map(|i| VariantMetadata {
                name: format!("x{i}"),
                orig_name: None,
                discr: None,
                fields: vec![],
                docstring: None,
            })
            .collect()
    }

    #[test]
    fn test_enum_with_many_variants() {
        check_metadata(
            &many_items::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_MANYVARIANTS,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::many_items".into(),
                name: "ManyVariants".into(),
                orig_name: None,
                shape: EnumShape::Enum,
                remote: false,
                discr_type: None,
                variants: many_variants_metadata(),
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_flat_error_with_many_variants() {
        check_metadata(
            &many_items::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ERROR_MANYVARIANTSERROR,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::many_items".into(),
                name: "ManyVariantsError".into(),
                orig_name: None,
                shape: EnumShape::Error { flat: true },
                remote: false,
                discr_type: None,
                variants: many_variants_metadata(),
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    fn many_fields_metadata() -> Vec<FieldMetadata> {
        (0..300)
            .map(|i| FieldMetadata {
                name: format!("x{i}"),
                orig_name: None,
                ty: Type::UInt8,
                default: None,
                docstring: None,
            })
            .collect()
    }

    #[test]
    fn test_record_with_many_fields() {
        check_metadata(
            &many_items::UNIFFI_META_UNIFFI_FIXTURE_METADATA_RECORD_MANYFIELDS,
            RecordMetadata {
                module_path: "uniffi_fixture_metadata::tests::many_items".into(),
                name: "ManyFields".into(),
                orig_name: None,
                remote: false,
                fields: many_fields_metadata(),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_enum_variant_with_many_fields() {
        check_metadata(
            &many_items::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ENUM_MANYVARIANTFIELDS,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::many_items".into(),
                name: "ManyVariantFields".into(),
                orig_name: None,
                shape: EnumShape::Enum,
                remote: false,
                discr_type: None,
                variants: vec![VariantMetadata {
                    name: "Variant".into(),
                    orig_name: None,
                    discr: None,
                    fields: many_fields_metadata(),
                    docstring: None,
                }],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_function_with_many_args() {
        let Metadata::Func(meta) =
            read_metadata(&many_items::UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_MANY_ARGS).unwrap()
        else {
            panic!("expected function metadata");
        };
        assert_eq!(meta.name, "many_args");
        assert_eq!(meta.return_type, None);
        assert_eq!(
            meta.inputs,
            (0..300)
                .map(|i| FnParamMetadata::simple(&format!("x{i}"), Type::UInt8))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_enum_method() {
        check_metadata(
            &state::UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_STATE_STATE_METHOD_RENAMED,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata::tests::state".into(),
                self_name: "State".into(),
                name: "state_method_renamed".into(),
                orig_name: Some("state_method".into()),
                is_async: false,
                inputs: vec![],
                return_type: None,
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(
                    state::uniffi_uniffi_fixture_metadata_checksum_method_state_state_method_renamed(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_simple_error() {
        check_metadata(
            &error::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ERROR_FLATERROR,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::error".into(),
                name: "FlatError".into(),
                orig_name: None,
                shape: EnumShape::Error { flat: true },
                remote: false,
                discr_type: None,
                variants: vec![
                    VariantMetadata {
                        name: "Overflow".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "DivideByZero".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                ],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_complex_error() {
        check_metadata(
            &error::UNIFFI_META_UNIFFI_FIXTURE_METADATA_ERROR_COMPLEXERROR,
            EnumMetadata {
                module_path: "uniffi_fixture_metadata::tests::error".into(),
                name: "ComplexError".into(),
                orig_name: None,
                shape: EnumShape::Error { flat: false },
                remote: false,
                discr_type: None,
                variants: vec![
                    VariantMetadata {
                        name: "NotFound".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "PermissionDenied".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![FieldMetadata {
                            name: "reason".into(),
                            orig_name: None,
                            ty: Type::String,
                            default: None,
                            docstring: None,
                        }],
                        docstring: None,
                    },
                    VariantMetadata {
                        name: "InvalidWeapon".into(),
                        orig_name: None,
                        discr: None,
                        fields: vec![FieldMetadata {
                            name: "weapon".into(),
                            orig_name: None,
                            ty: Type::Enum {
                                module_path: "uniffi_fixture_metadata::tests::weapon".into(),
                                name: "WeaponRenamed".into(),
                            },
                            default: None,
                            docstring: None,
                        }],
                        docstring: None,
                    },
                ],
                non_exhaustive: false,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_interface() {
        check_metadata(
            &calc::UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_CALCULATORRENAMED,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::calc".into(),
                name: "CalculatorRenamed".into(),
                orig_name: Some("Calculator".into()),
                remote: false,
                imp: ObjectImpl::Struct,
                docstring: None,
            },
        );
    }

    #[test]
    fn test_uniffi_traits_object() {
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIAL_DEBUG).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Debug { fmt })
                if fmt.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   fmt.self_name == "Special"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIAL_EQ).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Eq { eq, ne })
                if eq.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   eq.self_name == "Special" &&
                   ne.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   ne.self_name == "Special"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIAL_ORD).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Ord { cmp })
                if cmp.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   cmp.self_name == "Special"
        ));
    }

    #[test]
    fn test_uniffi_traits_enum() {
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALENUM_DEBUG).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Debug { fmt })
                if fmt.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   fmt.self_name == "SpecialEnum"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALENUM_EQ).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Eq { eq, ne })
                if eq.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   eq.self_name == "SpecialEnum" &&
                   ne.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   ne.self_name == "SpecialEnum"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALENUM_ORD).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Ord { cmp })
                if cmp.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   cmp.self_name == "SpecialEnum"
        ));
    }

    #[test]
    fn test_uniffi_traits_record() {
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALRECORD_DEBUG).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Debug { fmt })
                if fmt.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   fmt.self_name == "SpecialRecord"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALRECORD_EQ).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Eq { eq, ne })
                if eq.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   eq.self_name == "SpecialRecord" &&
                   ne.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   ne.self_name == "SpecialRecord"
        ));
        assert!(matches!(
            uniffi_meta::read_metadata(&uniffi_traits::UNIFFI_META_UNIFFI_FIXTURE_METADATA_UNIFFI_TRAIT_SPECIALRECORD_ORD).unwrap(),
            Metadata::UniffiTrait(UniffiTraitMetadata::Ord { cmp })
                if cmp.module_path == "uniffi_fixture_metadata::tests::uniffi_traits" &&
                   cmp.self_name == "SpecialRecord"
        ));
    }
}

mod test_function_metadata {
    use super::*;
    use std::sync::Arc;

    #[uniffi::export(name = "test_func_renamed")]
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
        #[uniffi::method(name = "display_result_renamed")]
        fn display_result(&self, val: String);
    }

    #[uniffi::export(name = "CalculatorRenamed")]
    impl Calculator {
        #[uniffi::constructor]
        pub fn new() -> Self {
            Self {}
        }

        #[uniffi::constructor(name = "new_renamed")]
        pub fn new2() -> Self {
            Self {}
        }

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

    #[uniffi::export(with_foreign)]
    pub trait TraitWithForeign: Send + Sync {
        fn test_method(&self, a: String, b: u32) -> String;
    }

    #[allow(unused)]
    #[uniffi::export]
    fn input_trait_with_foreign(val: Arc<dyn TraitWithForeign>) {}

    // Test that explicit `rust` flag produces TraitKind::RustOnly
    #[uniffi::export(rust)]
    pub trait TraitExplicitRust: Send + Sync {
        fn test_method(&self) -> String;
    }

    #[allow(unused)]
    #[uniffi::export]
    fn input_trait_explicit_rust(val: Arc<dyn TraitExplicitRust>) {}

    // Test the canonical `rust, foreign` bare-flag syntax (equivalent to `with_foreign`)
    #[uniffi::export(rust, foreign)]
    pub trait TraitImplsAny: Send + Sync {
        fn test_method(&self) -> String;
    }

    #[allow(unused)]
    #[uniffi::export]
    fn input_trait_impls_any(val: Arc<dyn TraitImplsAny>) {}

    // Test the canonical `foreign` bare-flag syntax (foreign-only, Arc-based)
    #[uniffi::export(foreign)]
    pub trait TraitImplsForeign: Send + Sync {
        fn test_method(&self) -> String;
    }

    #[allow(unused)]
    #[uniffi::export]
    fn input_trait_impls_foreign(val: Arc<dyn TraitImplsForeign>) {}

    #[test]
    fn test_function() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_RENAMED,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_func_renamed".into(),
                orig_name: Some("test_func".into()),
                is_async: false,
                inputs: vec![
                    FnParamMetadata::simple(
                        "person",
                        Type::Record {
                            module_path: "uniffi_fixture_metadata::tests::person".into(),
                            name: "PersonRenamed".into(),
                        },
                    ),
                    FnParamMetadata::simple(
                        "weapon",
                        Type::Enum {
                            module_path: "uniffi_fixture_metadata::tests::weapon".into(),
                            name: "WeaponRenamed".into(),
                        },
                    ),
                ],
                return_type: Some(Type::String),
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_RENAMED.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_function_no_return() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_func_no_return".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![],
                return_type: None,
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_function_that_throws() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_func_that_throws".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![],
                return_type: Some(Type::Enum {
                    module_path: "uniffi_fixture_metadata::tests::state".into(),
                    name: "State".into(),
                }),
                throws: Some(Type::Enum {
                    module_path: "uniffi_fixture_metadata::tests::error".into(),
                    name: "FlatError".into(),
                }),
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_THAT_THROWS.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_function_that_throws_no_return() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_func_no_return_that_throws".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![],
                return_type: None,
                throws: Some(Type::Enum {
                    module_path: "uniffi_fixture_metadata::tests::error".into(),
                    name: "FlatError".into(),
                }),
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_FUNC_NO_RETURN_THAT_THROWS
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_ADD,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                self_name: "CalculatorRenamed".into(),
                name: "add".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![
                    FnParamMetadata::simple("a", Type::UInt8),
                    FnParamMetadata::simple("b", Type::UInt8),
                ],
                return_type: Some(Type::UInt8),
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_ADD
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_constructor() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_CONSTRUCTOR_CALCULATORRENAMED_NEW,
            ConstructorMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                self_name: "CalculatorRenamed".into(),
                name: "new".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![],
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_CONSTRUCTOR_CALCULATORRENAMED_NEW
                        .checksum(),
                ),
                docstring: None,
            },
        );
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_CONSTRUCTOR_CALCULATORRENAMED_NEW_RENAMED,
            ConstructorMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                self_name: "CalculatorRenamed".into(),
                name: "new_renamed".into(),
                orig_name: Some("new2".into()),
                is_async: false,
                inputs: vec![],
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_CONSTRUCTOR_CALCULATORRENAMED_NEW_RENAMED
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_async_function() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_async_func".into(),
                orig_name: None,
                is_async: true,
                inputs: vec![
                    FnParamMetadata::simple(
                        "person",
                        Type::Record {
                            module_path: "uniffi_fixture_metadata::tests::person".into(),
                            name: "PersonRenamed".into(),
                        },
                    ),
                    FnParamMetadata::simple(
                        "weapon",
                        Type::Enum {
                            module_path: "uniffi_fixture_metadata::tests::weapon".into(),
                            name: "WeaponRenamed".into(),
                        },
                    ),
                ],
                return_type: Some(Type::String),
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_async_function_that_throws() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC_THAT_THROWS,
            FnMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "test_async_func_that_throws".into(),
                orig_name: None,
                is_async: true,
                inputs: vec![],
                return_type: Some(Type::Enum {
                    module_path: "uniffi_fixture_metadata::tests::state".into(),
                    name: "State".into(),
                }),
                throws: Some(Type::Enum {
                    module_path: "uniffi_fixture_metadata::tests::error".into(),
                    name: "FlatError".into(),
                }),
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_TEST_ASYNC_FUNC_THAT_THROWS
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_async_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_ASYNC_SUB,
            MethodMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                self_name: "CalculatorRenamed".into(),
                name: "async_sub".into(),
                orig_name: None,
                is_async: true,
                inputs: vec![
                    FnParamMetadata::simple("a", Type::UInt8),
                    FnParamMetadata::simple("b", Type::UInt8),
                ],
                return_type: Some(Type::UInt8),
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_ASYNC_SUB
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_metadata() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_CALCULATORDISPLAY,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "CalculatorDisplay".into(),
                orig_name: None,
                remote: false,
                imp: ObjectImpl::Trait(TraitKind::RustOnly),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_with_foreign_metadata() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_TRAITWITHFOREIGN,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "TraitWithForeign".into(),
                orig_name: None,
                remote: false,
                imp: ObjectImpl::Trait(TraitKind::Both),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_type_data() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_GET_DISPLAY,
            MethodMetadata {
                // The main point of this test is to check the `Type` value for a trait interface
                return_type: Some(Type::Object {
                    module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                    name: "CalculatorDisplay".into(),
                    imp: ObjectImpl::Trait(TraitKind::RustOnly),
                }),
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                self_name: "CalculatorRenamed".into(),
                name: "get_display".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![],
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORRENAMED_GET_DISPLAY
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_method() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORDISPLAY_DISPLAY_RESULT_RENAMED,
            TraitMethodMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                trait_name: "CalculatorDisplay".into(),
                index: 0,
                name: "display_result_renamed".into(),
                orig_name: Some("display_result".into()),
                is_async: false,
                inputs: vec![
                    FnParamMetadata::simple("val", Type::String),
                ],
                return_type: None,
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_CALCULATORDISPLAY_DISPLAY_RESULT_RENAMED
                    .checksum()),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_with_foreign_type_data() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_WITH_FOREIGN,
            FnMetadata {
                inputs: vec![
                    // The main point of this test is to check the `Type` value for a trait interface
                    FnParamMetadata::simple(
                        "val",
                        Type::Object {
                            module_path: "uniffi_fixture_metadata::tests::test_function_metadata"
                                .into(),
                            name: "TraitWithForeign".into(),
                            imp: ObjectImpl::Trait(TraitKind::Both),
                        },
                    ),
                ],
                // We might as well test other fields too though
                return_type: None,
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "input_trait_with_foreign".into(),
                orig_name: None,
                is_async: false,
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_WITH_FOREIGN
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_callback_interface() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_CALLBACK_INTERFACE_LOGGER,
            CallbackInterfaceMetadata {
                module_path: "uniffi_fixture_metadata::tests".into(),
                name: "Logger".into(),
                docstring: None,
            },
        );
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_METHOD_LOGGER_LOG,
            TraitMethodMetadata {
                module_path: "uniffi_fixture_metadata::tests".into(),
                trait_name: "Logger".into(),
                index: 0,
                name: "log".into(),
                orig_name: None,
                is_async: false,
                inputs: vec![FnParamMetadata::simple("message", Type::String)],
                return_type: None,
                throws: None,
                takes_self_by_arc: false,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_METHOD_LOGGER_LOG.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_explicit_rust_metadata() {
        // Explicit `rust` flag also produces TraitKind::RustOnly
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_TRAITEXPLICITRUST,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "TraitExplicitRust".into(),
                orig_name: None,
                remote: false,
                imp: ObjectImpl::Trait(TraitKind::RustOnly),
                docstring: None,
            },
        );
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_EXPLICIT_RUST,
            FnMetadata {
                inputs: vec![FnParamMetadata::simple(
                    "val",
                    Type::Object {
                        module_path: "uniffi_fixture_metadata::tests::test_function_metadata"
                            .into(),
                        name: "TraitExplicitRust".into(),
                        imp: ObjectImpl::Trait(TraitKind::RustOnly),
                    },
                )],
                return_type: None,
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "input_trait_explicit_rust".into(),
                orig_name: None,
                is_async: false,
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_EXPLICIT_RUST
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_impls_any_metadata() {
        // `rust, foreign` should produce the same metadata as `with_foreign`
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_TRAITIMPLSANY,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "TraitImplsAny".into(),
                orig_name: None,
                remote: false,
                imp: ObjectImpl::Trait(TraitKind::Both),
                docstring: None,
            },
        );
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_IMPLS_ANY,
            FnMetadata {
                inputs: vec![FnParamMetadata::simple(
                    "val",
                    Type::Object {
                        module_path: "uniffi_fixture_metadata::tests::test_function_metadata"
                            .into(),
                        name: "TraitImplsAny".into(),
                        imp: ObjectImpl::Trait(TraitKind::Both),
                    },
                )],
                return_type: None,
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "input_trait_impls_any".into(),
                orig_name: None,
                is_async: false,
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_IMPLS_ANY.checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_impls_foreign_metadata() {
        // `foreign` should produce the same metadata as the removed `foreign_only` flag
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_TRAITIMPLSFOREIGN,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "TraitImplsForeign".into(),
                orig_name: None,
                remote: false,
                imp: ObjectImpl::Trait(TraitKind::ForeignOnly),
                docstring: None,
            },
        );
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_IMPLS_FOREIGN,
            FnMetadata {
                inputs: vec![FnParamMetadata::simple(
                    "val",
                    Type::Object {
                        module_path: "uniffi_fixture_metadata::tests::test_function_metadata"
                            .into(),
                        name: "TraitImplsForeign".into(),
                        imp: ObjectImpl::Trait(TraitKind::ForeignOnly),
                    },
                )],
                return_type: None,
                module_path: "uniffi_fixture_metadata::tests::test_function_metadata".into(),
                name: "input_trait_impls_foreign".into(),
                orig_name: None,
                is_async: false,
                throws: None,
                checksum: Some(
                    UNIFFI_META_CONST_UNIFFI_FIXTURE_METADATA_FUNC_INPUT_TRAIT_IMPLS_FOREIGN
                        .checksum(),
                ),
                docstring: None,
            },
        );
    }

    #[test]
    fn test_trait_object_impl() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_INTERFACE_REALLOGGER,
            ObjectMetadata {
                module_path: "uniffi_fixture_metadata::tests".into(),
                name: "RealLogger".into(),
                orig_name: None,
                imp: ObjectImpl::Struct,
                remote: false,
                docstring: None,
            },
        );

        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_OBJECT_TRAIT_IMPL_REALLOGGER_LOGGER,
            ObjectTraitImplMetadata {
                ty: Type::Object {
                    module_path: "uniffi_fixture_metadata::tests".into(),
                    name: "RealLogger".into(),
                    imp: ObjectImpl::Struct,
                },
                trait_ty: Type::CallbackInterface {
                    module_path: "uniffi_fixture_metadata::tests".into(),
                    name: "Logger".into(),
                },
            },
        );
    }
}

mod test_custom_types {
    use super::*;

    pub struct CustomString(pub String);
    uniffi::custom_newtype!(CustomString, String);

    pub struct CustomPerson(pub Person);
    uniffi::custom_newtype!(CustomPerson, Person);

    #[test]
    fn test_custom_types() {
        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_CUSTOM_TYPE_CUSTOMSTRING,
            CustomTypeMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_custom_types".into(),
                name: "CustomString".into(),
                orig_name: None,
                builtin: Type::String,
                docstring: None,
            },
        );

        check_metadata(
            &UNIFFI_META_UNIFFI_FIXTURE_METADATA_CUSTOM_TYPE_CUSTOMPERSON,
            CustomTypeMetadata {
                module_path: "uniffi_fixture_metadata::tests::test_custom_types".into(),
                name: "CustomPerson".into(),
                orig_name: None,
                builtin: Type::Record {
                    module_path: "uniffi_fixture_metadata::tests::person".into(),
                    name: "PersonRenamed".into(),
                },
                docstring: None,
            },
        );
    }
}
