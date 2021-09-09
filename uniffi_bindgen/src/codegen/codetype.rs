/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::dispatch::*;
use crate::interface::FFIType;

type_dispatch! {
    /// Type behavior for generated code
    ///
    /// This trait handles the language-agnostic behavior.  We also create a language-specific
    /// version in the bindings/scaffolding code.
    pub trait NewCodeType {
        /// Get the canonical, unique-within-this-component name for a type.
        ///
        /// When generating helper code for foreign language bindings, this is used to generate
        /// class and function names for the type (for example `FFIConverter{canonical_name}`).
        /// We support this by defining a naming convention where each type gets a unique canonical
        /// name, constructed recursively from the names of its component types (if any).
        fn canonical_name(&self) -> String;

        /// FFI type that this type is lowered to
        fn ffi_type(&self) -> FFIType;
    }
}

// NewCodeType is simple enough that we can implement it in one file.

impl NewCodeType for SimpleTypeHandler {
    fn canonical_name(&self) -> String {
        match self {
            Self::Int8 => "I8".into(),
            Self::UInt8 => "U8".into(),
            Self::Int16 => "I16".into(),
            Self::UInt16 => "U16".into(),
            Self::Int32 => "I32".into(),
            Self::UInt32 => "U32".into(),
            Self::Int64 => "I64".into(),
            Self::UInt64 => "U64".into(),
            Self::Float32 => "F32".into(),
            Self::Float64 => "F64".into(),
            Self::String => "String".into(),
            Self::Boolean => "Bool".into(),
        }
    }

    fn ffi_type(&self) -> FFIType {
        match self {
            // Types that are the same map to themselves, naturally.
            Self::UInt8 => FFIType::UInt8,
            Self::Int8 => FFIType::Int8,
            Self::UInt16 => FFIType::UInt16,
            Self::Int16 => FFIType::Int16,
            Self::UInt32 => FFIType::UInt32,
            Self::Int32 => FFIType::Int32,
            Self::UInt64 => FFIType::UInt64,
            Self::Int64 => FFIType::Int64,
            Self::Float32 => FFIType::Float32,
            Self::Float64 => FFIType::Float64,
            // Booleans lower into an Int8, to work around a bug in JNA.
            Self::Boolean => FFIType::Int8,
            // Strings are always owned rust values.
            // We might add a separate type for borrowed strings in future.
            Self::String => FFIType::RustBuffer,
        }
    }
}

impl NewCodeType for RecordTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for EnumTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for ErrorTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for ObjectTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for CallbackInterfaceTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("CallbackInterface{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::UInt64
    }
}

impl NewCodeType for TimestampTypeHandler {
    fn canonical_name(&self) -> String {
        "Timestamp".into()
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for DurationTypeHandler {
    fn canonical_name(&self) -> String {
        "Duration".into()
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for OptionalTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Optional{}", self.inner.canonical_name())
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for SequenceTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Sequence{}", self.inner.canonical_name())
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for MapTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Map{}", self.inner.canonical_name())
    }

    fn ffi_type(&self) -> FFIType {
        FFIType::RustBuffer
    }
}

impl NewCodeType for ExternalTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        panic!("External type not handled yet");
    }
}

impl NewCodeType for WrapperTypeHandler<'_> {
    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn ffi_type(&self) -> FFIType {
        self.wrapped.ffi_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::ComponentInterface;

    #[test]
    fn test_canonical_name() {
        const UDL: &str = r#"
            namespace test{};
            interface TestObject { };
            dictionary TestRecord {
                u8 field1;
                string field2;
                timestamp? field3;
                sequence<TestObject>? field4;
            };
        "#;
        let ci = ComponentInterface::from_webidl(UDL).unwrap();
        // Non-exhaustive, but gives a bit of a flavour of what we want.
        let record = ci.get_record_definition("TestRecord").unwrap();
        let fields = record.fields();
        assert_eq!(record.type_().canonical_name(), "TypeTestRecord");
        assert_eq!(record.canonical_name(), "TypeTestRecord");
        assert_eq!(fields[0].type_().canonical_name(), "U8");
        assert_eq!(fields[1].type_().canonical_name(), "String");
        assert_eq!(fields[2].type_().canonical_name(), "OptionalTimestamp");
        assert_eq!(
            fields[3].type_().canonical_name(),
            "OptionalSequenceTypeTestObject"
        );
    }
}
