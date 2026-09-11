/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, spanned::Spanned, FnArg, Ident, ItemFn, Pat};

use crate::{
    attrs::{DefaultMap, FunctionAttributes},
    paths::LookupCache,
    ErrorKind::*,
    Ir, RPath, Result, Visibility,
};

#[derive(Clone)]
pub struct Function {
    pub attrs: FunctionAttributes,
    pub is_async: bool,
    pub ident: Ident,
    pub vis: Visibility,
    pub args: Vec<Argument>,
    pub return_ty: ReturnType,
}

#[derive(Clone)]
pub struct Argument {
    pub ident: Ident,
    pub ty: syn::Type,
}

#[derive(Clone)]
pub struct ReturnType {
    pub return_ty: syn::ReturnType,
}

impl Function {
    pub fn parse(attrs: FunctionAttributes, f: ItemFn) -> syn::Result<Self> {
        let (is_async, return_ty) =
            ReturnType::parse_async(f.sig.asyncness.is_some(), f.sig.output)?;
        Ok(Self {
            attrs,
            ident: f.sig.ident,
            vis: f.vis.into(),
            is_async,
            args: f
                .sig
                .inputs
                .into_iter()
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_ty,
        })
    }

    pub fn fn_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        item_path: RPath<'ir>,
    ) -> Result<uniffi_meta::FnMetadata> {
        let module_path = item_path.parent()?;
        let names = item_path.public_path_to_item(ir, cache)?;
        let (return_type, throws) =
            self.return_ty
                .return_type_and_throws(ir, cache, &module_path)?;

        Ok(uniffi_meta::FnMetadata {
            module_path: names.module_path,
            name: names.name,
            orig_name: names.orig_name,
            is_async: self.is_async,
            docstring: self.attrs.docstring.clone(),
            checksum: None,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_fn_metadata(ir, cache, &module_path, &self.attrs.defaults))
                .collect::<Result<Vec<_>>>()?,
            return_type,
            throws,
        })
    }
}

impl Argument {
    pub fn parse(arg: FnArg) -> syn::Result<Self> {
        let span = arg.span();
        let pat_ty = match arg {
            FnArg::Receiver(_) => return Err(syn::Error::new(span, InvalidArgType)),
            FnArg::Typed(pat_ty) => pat_ty,
        };
        let ident = match *pat_ty.pat {
            Pat::Ident(p) => p.ident,
            _ => return Err(syn::Error::new(span, InvalidArgType)),
        };
        Ok(Self {
            ident,
            ty: *pat_ty.ty,
        })
    }

    pub fn create_fn_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        defaults: &DefaultMap,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        self.create_metadata(ir, cache, path, defaults, None)
    }

    pub fn create_method_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        defaults: &DefaultMap,
        self_type: &uniffi_meta::Type,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        self.create_metadata(ir, cache, path, defaults, Some(self_type))
    }

    pub fn create_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        defaults: &DefaultMap,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        let arg = path.resolve_arg(ir, cache, &self.ty, self_ty)?;
        let default = defaults.get_uniffi_meta(path.file_id(), &self.ident, &arg.ty)?;

        Ok(uniffi_meta::FnParamMetadata {
            name: self.ident.unraw().to_string(),
            ty: arg.ty,
            pass_by: arg.pass_by,
            default,
            optional: false,
        })
    }
}

impl ReturnType {
    pub fn parse(return_ty: syn::ReturnType) -> syn::Result<Self> {
        Ok(Self { return_ty })
    }

    /// Determine whether a function is async and compute its effective return type.
    ///
    /// A function is async when it is declared `async fn` (`declared_async`), or when it
    /// manually returns a boxed future such as `Pin<Box<dyn Future<Output = T> + Send>>` —
    /// the shape `#[async_trait]` expands `async fn` into. Removing the `#[async_trait]`
    /// macro and the `async` modifier while returning that future by hand is still an async
    /// function as far as the FFI is concerned.
    ///
    /// For a real `async fn`, `syn` hands us the unsugared `Output` type directly. To treat
    /// the manual future the same way, we unwrap the future here and use its `Output` as the
    /// effective return type.
    pub fn parse_async(declared_async: bool, output: syn::ReturnType) -> syn::Result<(bool, Self)> {
        if !declared_async {
            if let syn::ReturnType::Type(arrow, ty) = &output {
                if let Some(inner) = uniffi_syn_utils::future_output_type(ty) {
                    return Ok((
                        true,
                        Self {
                            return_ty: syn::ReturnType::Type(*arrow, Box::new(inner)),
                        },
                    ));
                }
            }
        }
        Ok((declared_async, Self::parse(output)?))
    }

    pub fn return_type_and_throws<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        self._return_type_and_throws(ir, cache, module_path, None)
    }

    pub fn return_type_and_throws_for_method<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_type: &uniffi_meta::Type,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        self._return_type_and_throws(ir, cache, module_path, Some(self_type))
    }

    fn _return_type_and_throws<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        Ok(match &self.return_ty {
            syn::ReturnType::Default => (None, None),
            syn::ReturnType::Type(_, return_ty) => {
                let rt = module_path.resolve_return_type(ir, cache, return_ty, self_ty)?;
                (rt.ok, rt.err)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    /// Run `parse_async` on `-> #output` and return `(is_async, effective_return_type)`,
    /// with the return type rendered back to a normalized token string for comparison.
    fn parse_async(declared_async: bool, output: &str) -> (bool, Option<String>) {
        let output: syn::ReturnType = syn::parse_str(output).unwrap();
        let (is_async, rt) = ReturnType::parse_async(declared_async, output).unwrap();
        let rendered = match rt.return_ty {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
        };
        (is_async, rendered)
    }

    #[test]
    fn declared_async_is_left_untouched() {
        // A real `async fn`: `syn` already gives us the unsugared `Output`, so the return
        // type must pass through verbatim.
        assert_eq!(
            parse_async(true, "-> String"),
            (true, Some("String".into()))
        );
        assert_eq!(parse_async(true, ""), (true, None));
    }

    #[test]
    fn plain_return_types_are_not_async() {
        assert_eq!(
            parse_async(false, "-> String"),
            (false, Some("String".into()))
        );
        assert_eq!(parse_async(false, ""), (false, None));
        // A future nested as an argument, not the future itself, must not be unwrapped.
        assert_eq!(
            parse_async(false, "-> Vec<u8>"),
            (false, Some("Vec < u8 >".into()))
        );
    }

    #[test]
    fn boxed_future_is_detected_and_unwrapped() {
        // The shape `#[async_trait]` expands `async fn ... -> String` into.
        assert_eq!(
            parse_async(
                false,
                "-> Pin<Box<dyn Future<Output = String> + Send + 'a>>"
            ),
            (true, Some("String".into()))
        );
        // Fully-qualified paths for `Pin` and `Future`.
        assert_eq!(
            parse_async(
                false,
                "-> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = u8> + Send>>"
            ),
            (true, Some("u8".into()))
        );
        // `Output = ()` unwraps to the unit type (still `Some`, resolved as void downstream).
        assert_eq!(
            parse_async(false, "-> Pin<Box<dyn Future<Output = ()> + Send>>"),
            (true, Some("()".into()))
        );
        // Without the `Pin` wrapper we don't consider it's compatible.
        assert_eq!(
            parse_async(false, "-> Box<dyn Future<Output = Arc<Self>>>"),
            (
                false,
                Some("Box < dyn Future < Output = Arc < Self > > >".into())
            )
        );
        // `impl Future` won't work either.
        assert_eq!(
            parse_async(false, "-> impl Future<Output = i64>"),
            (false, Some("impl Future < Output = i64 >".into()))
        );
    }
}
