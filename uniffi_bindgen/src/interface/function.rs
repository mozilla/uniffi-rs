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
use std::hash::{Hash, Hasher};

use anyhow::{bail, Result};

use super::attributes::{ArgumentAttributes, FunctionAttributes};
use super::ffi::{FFIArgument, FFIFunction};
use super::literal::{convert_default_value, Literal};
use super::types::Type;
use super::{APIConverter, ComponentInterface};

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone, Default)]
pub struct Function {
    pub(super) name: String,
    pub(super) arguments: Vec<Argument>,
    pub(super) return_type: Option<Type>,
    pub(super) ffi_func: FFIFunction,
    pub(super) attributes: FunctionAttributes,
    pub(super) docs: Vec<String>,
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

    pub fn docs(&self) -> Vec<&str> {
        self.docs.iter().map(|s| s.as_str()).collect()
    }

    pub fn ffi_func(&self) -> &FFIFunction {
        &self.ffi_func
    }

    pub fn throws(&self) -> Option<&str> {
        self.attributes.get_throws_err()
    }

    pub fn derive_ffi_func(&mut self, ci_prefix: &str) -> Result<()> {
        self.ffi_func.name.push_str(ci_prefix);
        self.ffi_func.name.push('_');
        self.ffi_func.name.push_str(&self.name);
        self.ffi_func.arguments = self.arguments.iter().map(|arg| arg.into()).collect();
        self.ffi_func.return_type = self.return_type.as_ref().map(|rt| rt.into());
        Ok(())
    }

    // Intentionally exactly the same as the Method version
    pub fn contains_unsigned_types(&self, ci: &ComponentInterface) -> bool {
        let check_return_type = {
            match self.return_type() {
                None => false,
                Some(t) => ci.type_contains_unsigned_types(t),
            }
        };
        check_return_type
            || self
                .arguments()
                .iter()
                .any(|&arg| ci.type_contains_unsigned_types(&arg.type_()))
    }
}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We don't include the FFIFunc in the hash calculation, because:
        //  - it is entirely determined by the other fields,
        //    so excluding it is safe.
        //  - its `name` property includes a checksum derived from  the very
        //    hash value we're trying to calculate here, so excluding it
        //    avoids a weird circular depenendency in the calculation.
        self.name.hash(state);
        self.arguments.hash(state);
        self.return_type.hash(state);
        self.attributes.hash(state);
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
            attributes: FunctionAttributes::try_from(self.attributes.as_ref())?,
            ..Default::default()
        })
    }
}

impl APIConverter<Function> for &syn::ItemFn {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Function> {
        let attrs = super::synner::Attributes::try_from(&self.attrs)?;
        // TODO: check a bunch of stuff, e.g. no async, no unsafe, no generics.
        if self.sig.variadic.is_some() {
            bail!("Variadic functions are not supported");
        }
        let mut attributes: Vec<super::attributes::Attribute> = vec![];
        let return_type = match &self.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, type_) => Some({
                let (throws, returns) = super::synner::destructure_if_result_type(type_)?;
                if let Some(err) = throws {
                    attributes.push(super::attributes::Attribute::Throws(err));
                }
                returns
            }),
        };
        let return_type = match return_type {
            None => None,
            Some(syn::Type::Tuple(t)) if t.elems.is_empty() => None,
            Some(t) => Some(ci.resolve_type_expression(t)?),
        };
        Ok(Function {
            name: self.sig.ident.to_string(),
            arguments: self
                .sig
                .inputs
                .iter()
                .map(|arg| arg.convert(ci))
                .collect::<Result<Vec<_>>>()?,
            return_type,
            attributes: super::attributes::FunctionAttributes::new(attributes),
            docs: attrs.docs,
            ..Default::default()
        })
    }
}

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Hash)]
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
    pub fn type_(&self) -> Type {
        self.type_.clone()
    }
    pub fn by_ref(&self) -> bool {
        self.by_ref
    }
    pub fn default_value(&self) -> Option<Literal> {
        self.default.clone()
    }
}

impl From<&Argument> for FFIArgument {
    fn from(a: &Argument) -> FFIArgument {
        FFIArgument {
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

impl APIConverter<Argument> for &syn::FnArg {
    fn convert(&self, ci: &mut ComponentInterface) -> Result<Argument> {
        let mut by_ref = false;
        let (name, type_) = match self {
            syn::FnArg::Receiver(_) => bail!("Cannot convert `self` arguments"),
            syn::FnArg::Typed(p) => {
                let name = super::synner::name_from_pattern(&p.pat)?;
                if name == "self" {
                    bail!("Cannot convert `self` arguments");
                }
                let type_ = match *p.ty {
                    syn::Type::Reference(ref rt) => {
                        by_ref = true;
                        ci.resolve_type_expression(&*rt.elem)?
                    }
                    ref t => ci.resolve_type_expression(&*t)?,
                };
                (name, type_)
            }
        };
        if let Type::Object(nm) = type_ {
            bail!("Objects cannot currently be passed as arguments: {}", nm);
        }
        Ok(Argument {
            name,
            type_,
            by_ref,
            optional: false,
            default: None,
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
        assert!(func1.throws().is_none());
        assert_eq!(func1.arguments().len(), 0);

        let func2 = ci.get_function_definition("rich").unwrap();
        assert_eq!(func2.name(), "rich");
        assert_eq!(
            func2.return_type().unwrap().canonical_name(),
            "SequenceOptionalstring"
        );
        assert!(matches!(func2.throws(), Some("TestError")));
        assert_eq!(func2.arguments().len(), 2);
        assert_eq!(func2.arguments()[0].name(), "arg1");
        assert_eq!(func2.arguments()[0].type_().canonical_name(), "u32");
        assert_eq!(func2.arguments()[1].name(), "arg2");
        assert_eq!(
            func2.arguments()[1].type_().canonical_name(),
            "RecordTestDict"
        );
        Ok(())
    }
}
