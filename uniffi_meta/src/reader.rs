/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::*;
use anyhow::{bail, Context, Result};

/// Metadata constants, make sure to keep this in sync with copy in `uniffi_core::metadata`
#[allow(dead_code)]
pub mod codes {
    // Top-level metadata item codes
    pub const FUNC: u8 = 0;
    pub const METHOD: u8 = 1;
    pub const RECORD: u8 = 2;
    pub const ENUM: u8 = 3;
    pub const INTERFACE: u8 = 4;
    pub const ERROR: u8 = 5;
    pub const NAMESPACE: u8 = 6;
    pub const UNKNOWN: u8 = 255;

    // Type codes
    pub const TYPE_U8: u8 = 0;
    pub const TYPE_U16: u8 = 1;
    pub const TYPE_U32: u8 = 2;
    pub const TYPE_U64: u8 = 3;
    pub const TYPE_I8: u8 = 4;
    pub const TYPE_I16: u8 = 5;
    pub const TYPE_I32: u8 = 6;
    pub const TYPE_I64: u8 = 7;
    pub const TYPE_F32: u8 = 8;
    pub const TYPE_F64: u8 = 9;
    pub const TYPE_BOOL: u8 = 10;
    pub const TYPE_STRING: u8 = 11;
    pub const TYPE_OPTION: u8 = 12;
    pub const TYPE_RECORD: u8 = 13;
    pub const TYPE_ENUM: u8 = 14;
    pub const TYPE_ERROR: u8 = 15;
    pub const TYPE_INTERFACE: u8 = 16;
    pub const TYPE_VEC: u8 = 17;
    pub const TYPE_HASH_MAP: u8 = 18;
    pub const TYPE_SYSTEM_TIME: u8 = 19;
    pub const TYPE_DURATION: u8 = 20;
    pub const TYPE_CALLBACK_INTERFACE: u8 = 21;
    pub const TYPE_CUSTOM: u8 = 22;
    pub const TYPE_RESULT: u8 = 23;
    pub const TYPE_UNIT: u8 = 255;
}

/// Trait for types that can read Metadata
///
/// We implement this on &[u8] byte buffers
pub trait MetadataReader {
    fn read_u8(&mut self) -> Result<u8>;
    fn peek_u8(&mut self) -> Result<u8>;
    fn read_bool(&mut self) -> Result<bool>;
    fn read_string(&mut self) -> Result<String>;
    fn read_type(&mut self) -> Result<Type>;
    fn read_optional_type(&mut self) -> Result<Option<Type>>;
    fn read_return_type(&mut self) -> Result<(Option<Type>, Option<Type>)>;
    fn read_metadata(&mut self) -> Result<Metadata>;
    fn read_func(&mut self) -> Result<FnMetadata>;
    fn read_method(&mut self) -> Result<MethodMetadata>;
    fn read_record(&mut self) -> Result<RecordMetadata>;
    fn read_enum(&mut self) -> Result<EnumMetadata>;
    fn read_error(&mut self) -> Result<ErrorMetadata>;
    fn read_object(&mut self) -> Result<ObjectMetadata>;
    fn read_fields(&mut self) -> Result<Vec<FieldMetadata>>;
    fn read_variants(&mut self) -> Result<Vec<VariantMetadata>>;
    fn read_flat_variants(&mut self) -> Result<Vec<VariantMetadata>>;
    fn read_inputs(&mut self) -> Result<Vec<FnParamMetadata>>;
}

impl MetadataReader for &[u8] {
    fn read_u8(&mut self) -> Result<u8> {
        if !self.is_empty() {
            let value = self[0];
            *self = &self[1..];
            Ok(value)
        } else {
            bail!("Buffer is empty")
        }
    }

    fn peek_u8(&mut self) -> Result<u8> {
        if !self.is_empty() {
            Ok(self[0])
        } else {
            bail!("Buffer is empty")
        }
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? == 1)
    }

    fn read_string(&mut self) -> Result<String> {
        let size = self.read_u8()? as usize;
        let slice;
        (slice, *self) = self.split_at(size);
        String::from_utf8(slice.into()).context("Invalid string data")
    }

    fn read_type(&mut self) -> Result<Type> {
        let value = self.read_u8()?;
        Ok(match value {
            codes::TYPE_U8 => Type::U8,
            codes::TYPE_I8 => Type::I8,
            codes::TYPE_U16 => Type::U16,
            codes::TYPE_I16 => Type::I16,
            codes::TYPE_U32 => Type::U32,
            codes::TYPE_I32 => Type::I32,
            codes::TYPE_U64 => Type::U64,
            codes::TYPE_I64 => Type::I64,
            codes::TYPE_F32 => Type::F32,
            codes::TYPE_F64 => Type::F64,
            codes::TYPE_BOOL => Type::Bool,
            codes::TYPE_STRING => Type::String,
            codes::TYPE_DURATION => Type::Duration,
            codes::TYPE_SYSTEM_TIME => Type::SystemTime,
            codes::TYPE_RECORD => Type::Record {
                name: self.read_string()?,
            },
            codes::TYPE_ENUM => Type::Enum {
                name: self.read_string()?,
            },
            codes::TYPE_ERROR => Type::Error {
                name: self.read_string()?,
            },
            codes::TYPE_INTERFACE => Type::ArcObject {
                object_name: self.read_string()?,
            },
            codes::TYPE_CALLBACK_INTERFACE => Type::CallbackInterface {
                name: self.read_string()?,
            },
            codes::TYPE_CUSTOM => Type::Custom {
                name: self.read_string()?,
                builtin: Box::new(self.read_type()?),
            },
            codes::TYPE_OPTION => Type::Option {
                inner_type: Box::new(self.read_type()?),
            },
            codes::TYPE_VEC => Type::Vec {
                inner_type: Box::new(self.read_type()?),
            },
            codes::TYPE_HASH_MAP => Type::HashMap {
                key_type: Box::new(self.read_type()?),
                value_type: Box::new(self.read_type()?),
            },
            codes::TYPE_UNIT => bail!("Unexpected TYPE_UNIT"),
            codes::TYPE_RESULT => bail!("Unexpected TYPE_RESULT"),
            _ => bail!("Unexpected metadata type code: {value:?}"),
        })
    }

    fn read_optional_type(&mut self) -> Result<Option<Type>> {
        Ok(match self.peek_u8()? {
            codes::TYPE_UNIT => {
                _ = self.read_u8();
                None
            }
            _ => Some(self.read_type()?),
        })
    }

    fn read_return_type(&mut self) -> Result<(Option<Type>, Option<Type>)> {
        Ok(match self.peek_u8()? {
            codes::TYPE_UNIT => {
                _ = self.read_u8();
                (None, None)
            }
            codes::TYPE_RESULT => {
                _ = self.read_u8();
                (self.read_optional_type()?, self.read_optional_type()?)
            }
            _ => (Some(self.read_type()?), None),
        })
    }

    fn read_metadata(&mut self) -> Result<Metadata> {
        let value = self.read_u8()?;
        Ok(match value {
            codes::NAMESPACE => NamespaceMetadata {
                crate_name: self.read_string()?,
                name: self.read_string()?,
            }
            .into(),
            codes::FUNC => self.read_func()?.into(),
            codes::METHOD => self.read_method()?.into(),
            codes::RECORD => self.read_record()?.into(),
            codes::ENUM => self.read_enum()?.into(),
            codes::ERROR => self.read_error()?.into(),
            codes::INTERFACE => self.read_object()?.into(),
            _ => bail!("Unexpected metadata code: {value:?}"),
        })
    }

    fn read_func(&mut self) -> Result<FnMetadata> {
        let module_path = self.read_string()?;
        let name = self.read_string()?;
        let is_async = self.read_bool()?;
        let inputs = self.read_inputs()?;
        let (return_type, throws) = self.read_return_type()?;
        Ok(FnMetadata {
            module_path,
            name,
            is_async,
            inputs,
            return_type,
            throws,
        })
    }

    fn read_method(&mut self) -> Result<MethodMetadata> {
        let module_path = self.read_string()?;
        let self_name = self.read_string()?;
        let name = self.read_string()?;
        let is_async = self.read_bool()?;
        let inputs = self.read_inputs()?;
        let (return_type, throws) = self.read_return_type()?;
        Ok(MethodMetadata {
            module_path,
            self_name,
            name,
            is_async,
            inputs,
            return_type,
            throws,
        })
    }

    fn read_record(&mut self) -> Result<RecordMetadata> {
        Ok(RecordMetadata {
            module_path: self.read_string()?,
            name: self.read_string()?,
            fields: self.read_fields()?,
        })
    }

    fn read_enum(&mut self) -> Result<EnumMetadata> {
        Ok(EnumMetadata {
            module_path: self.read_string()?,
            name: self.read_string()?,
            variants: self.read_variants()?,
        })
    }

    fn read_error(&mut self) -> Result<ErrorMetadata> {
        let module_path = self.read_string()?;
        let name = self.read_string()?;
        let flat = self.read_bool()?;
        Ok(ErrorMetadata {
            module_path,
            name,
            flat,
            variants: if flat {
                self.read_flat_variants()?
            } else {
                self.read_variants()?
            },
        })
    }

    fn read_object(&mut self) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            module_path: self.read_string()?,
            name: self.read_string()?,
        })
    }

    fn read_fields(&mut self) -> Result<Vec<FieldMetadata>> {
        let len = self.read_u8()?;
        (0..len)
            .into_iter()
            .map(|_| {
                Ok(FieldMetadata {
                    name: self.read_string()?,
                    ty: self.read_type()?,
                })
            })
            .collect()
    }

    fn read_variants(&mut self) -> Result<Vec<VariantMetadata>> {
        let len = self.read_u8()?;
        (0..len)
            .into_iter()
            .map(|_| {
                Ok(VariantMetadata {
                    name: self.read_string()?,
                    fields: self.read_fields()?,
                })
            })
            .collect()
    }

    fn read_flat_variants(&mut self) -> Result<Vec<VariantMetadata>> {
        let len = self.read_u8()?;
        (0..len)
            .into_iter()
            .map(|_| {
                Ok(VariantMetadata {
                    name: self.read_string()?,
                    fields: vec![],
                })
            })
            .collect()
    }

    fn read_inputs(&mut self) -> Result<Vec<FnParamMetadata>> {
        let len = self.read_u8()?;
        (0..len)
            .into_iter()
            .map(|_| {
                Ok(FnParamMetadata {
                    name: self.read_string()?,
                    ty: self.read_type()?,
                })
            })
            .collect()
    }
}
