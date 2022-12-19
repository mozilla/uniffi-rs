/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Function definitions for a `ComponentInterface`.
//!
//! This module converts function definitions from UDL into structures that
//! can be added to a `ComponentInterface`. A declaration in the UDL like this:
//!
//! ```
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! namespace example {
//!     string hello();
//! };
//! # "##)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Will result in a [`Function`] member being added to the resulting [`ComponentInterface`]:
//!
//! ```
//! # use uniffi_bindgen::interface::Type;
//! # let ci = uniffi_bindgen::interface::ComponentInterface::from_webidl(r##"
//! # namespace example {
//! #     string hello();
//! # };
//! # "##)?;
//! let func = ci.get_function_definition("hello").unwrap();
//! assert_eq!(func.name(), "hello");
//! assert!(matches!(func.return_type(), Some(Type::String)));
//! assert_eq!(func.arguments().len(), 0);
//! # Ok::<(), anyhow::Error>(())
//! ```
use std::convert::TryFrom;

use anyhow::{bail, Result};
use uniffi_meta::Checksum;

use super::attributes::{ArgumentAttributes, Attribute, FunctionAttributes};
use super::ffi::{FfiArgument, FfiFunction};
use super::literal::{convert_default_value, Literal};
use super::types::{Type, TypeIterator};
use super::{convert_type, APIConverter, ComponentInterface};

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone, Checksum)]
pub struct Function {
    pub(super) name: String,
    pub(super) arguments: Vec<Argument>,
    pub(super) return_type: Option<Type>,
    // We don't include the FFIFunc in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from  the very
    //    hash value we're trying to calculate here, so excluding it
    //    avoids a weird circular dependency in the calculation.
    #[checksum_ignore]
    pub(super) ffi_func: FfiFunction,
    pub(super) attributes: FunctionAttributes,
}

impl Function {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }

    pub fn full_arguments(&self) -> Vec<Argument> {
        self.arguments.to_vec()
    }

    pub fn return_type(&self) -> Option<&Type> {
        self.return_type.as_ref()
    }

    pub fn ffi_func(&self) -> &FfiFunction {
        &self.ffi_func
    }

    pub fn throws(&self) -> bool {
        self.attributes.get_throws_err().is_some()
    }

    pub fn throws_name(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    pub fn throws_type(&self) -> Option<Type> {
        self.attributes
            .get_throws_err()
            .map(|name| Type::Error(name.to_owned()))
    }

    pub fn derive_ffi_func(&mut self, ci_prefix: &str) -> Result<()> {
        // The name is already set if the function is defined through a proc-macro invocation
        // rather than in UDL. Don't overwrite it in that case.
        if self.ffi_func.name.is_empty() {
            self.ffi_func.name = format!("{ci_prefix}_{}", self.name);
        }

        self.ffi_func.arguments = self.arguments.iter().map(|arg| arg.into()).collect();
        self.ffi_func.return_type = self.return_type.as_ref().map(|rt| rt.into());
        Ok(())
    }
}

impl From<uniffi_meta::FnParamMetadata> for Argument {
    fn from(meta: uniffi_meta::FnParamMetadata) -> Self {
        Argument {
            name: meta.name,
            type_: convert_type(&meta.ty),
            by_ref: false,
            optional: false,
            default: None,
        }
    }
}

impl From<uniffi_meta::FnMetadata> for Function {
    fn from(meta: uniffi_meta::FnMetadata) -> Self {
        let ffi_name = meta.ffi_symbol_name();

        let return_type = meta.return_type.map(|out| convert_type(&out));
        let arguments = meta.inputs.into_iter().map(Into::into).collect();

        let ffi_func = FfiFunction {
            name: ffi_name,
            ..FfiFunction::default()
        };

        Self {
            name: meta.name,
            arguments,
            return_type,
            ffi_func,
            attributes: meta.throws.map(Attribute::Throws).into_iter().collect(),
        }
    }
}

impl APIConverter<Function> for weedle::namespace::NamespaceMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Function> {
        match self {
            weedle::namespace::NamespaceMember::Operation(f) => f.convert(ci),
            _ => bail!("no support for namespace member type {:?} yet", self),
        }
    }
}

impl APIConverter<Function> for weedle::namespace::OperationNamespaceMember<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Function> {
        let return_type = ci.resolve_return_type_expression(&self.return_type)?;
        Ok(Function {
            name: match self.identifier {
                None => bail!("anonymous functions are not supported {:?}", self),
                Some(id) => id.0.to_string(),
            },
            return_type,
            arguments: self.args.body.list.convert(ci)?,
            ffi_func: Default::default(),
            attributes: FunctionAttributes::try_from(self.attributes.as_ref())?,
        })
    }
}

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Checksum)]
pub struct Argument {
    pub(super) name: String,
    pub(super) type_: Type,
    pub(super) by_ref: bool,
    pub(super) optional: bool,
    pub(super) default: Option<Literal>,
}

impl Argument {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn by_ref(&self) -> bool {
        self.by_ref
    }

    pub fn default_value(&self) -> Option<&Literal> {
        self.default.as_ref()
    }

    pub fn iter_types(&self) -> TypeIterator<'_> {
        self.type_.iter_types()
    }
}

impl From<&Argument> for FfiArgument {
    fn from(a: &Argument) -> FfiArgument {
        FfiArgument {
            name: a.name.clone(),
            type_: (&a.type_).into(),
        }
    }
}

impl APIConverter<Argument> for weedle::argument::Argument<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Argument> {
        match self {
            weedle::argument::Argument::Single(t) => t.convert(ci),
            weedle::argument::Argument::Variadic(_) => bail!("variadic arguments not supported"),
        }
    }
}

impl APIConverter<Argument> for weedle::argument::SingleArgument<'_> {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Argument> {
        let type_ = ci.resolve_type_expression(&self.type_)?;
        let default = match self.default {
            None => None,
            Some(v) => Some(convert_default_value(&v.value, &type_)?),
        };
        let by_ref = ArgumentAttributes::try_from(self.attributes.as_ref())?.by_ref();
        Ok(Argument {
            name: self.identifier.0.to_string(),
            type_,
            by_ref,
            optional: self.optional.is_some(),
            default,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_minimal_and_rich_function() -> Result<()> {
        let ci = ComponentInterface::from_webidl(
            r##"
            namespace test {
                void minimal();
                [Throws=TestError]
                sequence<string?> rich(u32 arg1, TestDict arg2);
            };
            [Error]
            enum TestError { "err" };
            dictionary TestDict {
                u32 field;
            };
        "##,
        )?;

        let func1 = ci.get_function_definition("minimal").unwrap();
        assert_eq!(func1.name(), "minimal");
        assert!(func1.return_type().is_none());
        assert!(func1.throws_type().is_none());
        assert_eq!(func1.arguments().len(), 0);

        let func2 = ci.get_function_definition("rich").unwrap();
        assert_eq!(func2.name(), "rich");
        assert_eq!(
            func2.return_type().unwrap().canonical_name(),
            "SequenceOptionalstring"
        );
        assert!(matches!(func2.throws_type(), Some(Type::Error(s)) if s == "TestError"));
        assert_eq!(func2.arguments().len(), 2);
        assert_eq!(func2.arguments()[0].name(), "arg1");
        assert_eq!(func2.arguments()[0].type_().canonical_name(), "u32");
        assert_eq!(func2.arguments()[1].name(), "arg2");
        assert_eq!(
            func2.arguments()[1].type_().canonical_name(),
            "TypeTestDict"
        );
        Ok(())
    }
}
