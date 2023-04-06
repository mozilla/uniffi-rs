/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    visit_mut::VisitMut,
    Attribute, Item, Path, Token, Type,
};

#[cfg(not(feature = "nightly"))]
pub fn mod_path() -> syn::Result<String> {
    // Without the nightly feature and TokenStream::expand_expr, just return the crate name

    use std::path::Path;

    use fs_err as fs;
    use once_cell::sync::Lazy;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CargoToml {
        package: Package,
        #[serde(default)]
        lib: Lib,
    }

    #[derive(Deserialize)]
    struct Package {
        name: String,
    }

    #[derive(Default, Deserialize)]
    struct Lib {
        name: Option<String>,
    }

    static LIB_CRATE_MOD_PATH: Lazy<Result<String, String>> = Lazy::new(|| {
        let manifest_dir =
            std::env::var_os("CARGO_MANIFEST_DIR").ok_or("`CARGO_MANIFEST_DIR` is not set")?;

        let cargo_toml_bytes =
            fs::read(Path::new(&manifest_dir).join("Cargo.toml")).map_err(|e| e.to_string())?;
        let cargo_toml = toml::from_slice::<CargoToml>(&cargo_toml_bytes)
            .map_err(|e| format!("Failed to parse `Cargo.toml`: {e}"))?;

        let lib_crate_name = cargo_toml
            .lib
            .name
            .unwrap_or_else(|| cargo_toml.package.name.replace('-', "_"));

        Ok(lib_crate_name)
    });

    LIB_CRATE_MOD_PATH
        .clone()
        .map_err(|e| syn::Error::new(Span::call_site(), e))
}

#[cfg(feature = "nightly")]
pub fn mod_path() -> syn::Result<String> {
    use proc_macro::TokenStream;

    let module_path_invoc = TokenStream::from(quote! { ::core::module_path!() });
    // We ask the compiler what `module_path!()` expands to here.
    // This is a nightly feature, tracked at https://github.com/rust-lang/rust/issues/90765
    let expanded_module_path = TokenStream::expand_expr(&module_path_invoc)
        .map_err(|e| syn::Error::new(Span::call_site(), e))?;
    Ok(syn::parse::<syn::LitStr>(expanded_module_path)?.value())
}

/// Rewrite Self type alias usage in an impl block to the type itself.
///
/// For example,
///
/// ```ignore
/// impl some::module::Foo {
///     fn method(
///         self: Arc<Self>,
///         arg: Option<Bar<(), Self>>,
///     ) -> Result<Self, Error> {
///         todo!()
///     }
/// }
/// ```
///
/// will be rewritten to
///
///  ```ignore
/// impl some::module::Foo {
///     fn method(
///         self: Arc<some::module::Foo>,
///         arg: Option<Bar<(), some::module::Foo>>,
///     ) -> Result<some::module::Foo, Error> {
///         todo!()
///     }
/// }
/// ```
pub fn rewrite_self_type(item: &mut Item) {
    let item = match item {
        Item::Impl(i) => i,
        _ => return,
    };

    struct RewriteSelfVisitor<'a>(&'a Type);

    impl<'a> VisitMut for RewriteSelfVisitor<'a> {
        fn visit_type_mut(&mut self, i: &mut Type) {
            match i {
                Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self") => {
                    *i = self.0.clone();
                }
                _ => syn::visit_mut::visit_type_mut(self, i),
            }
        }
    }

    let mut visitor = RewriteSelfVisitor(&item.self_ty);
    for item in &mut item.items {
        visitor.visit_impl_item_mut(item);
    }
}

pub fn try_read_field(f: &syn::Field) -> TokenStream {
    let ident = &f.ident;
    let ty = &f.ty;

    quote! {
        #ident: <#ty as ::uniffi::FfiConverter<crate::UniFfiTag>>::try_read(buf)?,
    }
}

pub fn ident_to_string(ident: &Ident) -> String {
    ident.unraw().to_string()
}

pub fn crate_name() -> String {
    std::env::var("CARGO_CRATE_NAME").unwrap().replace('-', "_")
}

pub fn create_metadata_items(
    kind: &str,
    name: &str,
    metadata_expr: TokenStream,
    checksum_fn_name: Option<String>,
) -> TokenStream {
    let crate_name = crate_name();
    let crate_name_upper = crate_name.to_uppercase();
    let kind_upper = kind.to_uppercase();
    let name_upper = name.to_uppercase();
    let const_ident =
        format_ident!("UNIFFI_META_CONST_{crate_name_upper}_{kind_upper}_{name_upper}");
    let static_ident = format_ident!("UNIFFI_META_{crate_name_upper}_{kind_upper}_{name_upper}");

    let checksum_fn = checksum_fn_name.map(|name| {
        let ident = Ident::new(&name, Span::call_site());
        quote! {
            #[doc(hidden)]
            #[no_mangle]
            pub extern "C" fn #ident() -> u16 {
                #const_ident.checksum()
            }
        }
    });

    quote! {
        const #const_ident: ::uniffi::MetadataBuffer = #metadata_expr;
        #[no_mangle]
        #[doc(hidden)]
        pub static #static_ident: [u8; #const_ident.size] = #const_ident.into_array();

        #checksum_fn
    }
}

pub fn try_metadata_value_from_usize(value: usize, error_message: &str) -> syn::Result<u8> {
    value
        .try_into()
        .map_err(|_| syn::Error::new(Span::call_site(), error_message))
}

pub fn chain<T>(
    a: impl IntoIterator<Item = T>,
    b: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = T> {
    a.into_iter().chain(b)
}

pub trait UniffiAttribute: Default {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self>;
    fn merge(self, other: Self) -> syn::Result<Self>;
}

pub fn parse_comma_separated<T: UniffiAttribute>(input: ParseStream<'_>) -> syn::Result<T> {
    let punctuated = input.parse_terminated::<T, Token![,]>(T::parse_one)?;
    punctuated.into_iter().try_fold(T::default(), T::merge)
}

#[derive(Default)]
struct AttributeNotAllowedHere;

impl UniffiAttribute for AttributeNotAllowedHere {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        Err(syn::Error::new(
            input.span(),
            "UniFFI attributes are not currently recognized in this position",
        ))
    }

    fn merge(self, _other: Self) -> syn::Result<Self> {
        Ok(Self)
    }
}

pub trait AttributeSliceExt {
    fn parse_uniffi_attributes<T: UniffiAttribute>(&self) -> syn::Result<T>;
    fn attributes_not_allowed_here(&self) -> Option<syn::Error>;
}

impl AttributeSliceExt for [Attribute] {
    fn parse_uniffi_attributes<T: UniffiAttribute>(&self) -> syn::Result<T> {
        self.iter()
            .filter(|attr| attr.path.is_ident("uniffi"))
            .try_fold(T::default(), |res, attr| {
                let parsed = attr.parse_args_with(parse_comma_separated)?;
                res.merge(parsed)
            })
    }

    fn attributes_not_allowed_here(&self) -> Option<syn::Error> {
        self.parse_uniffi_attributes::<AttributeNotAllowedHere>()
            .err()
    }
}

pub fn either_attribute_arg<T: Spanned>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
    match (a, b) {
        (None, None) => Ok(None),
        (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
        (Some(a), Some(b)) => {
            let mut error = syn::Error::new(a.span(), "redundant attribute argument");
            error.combine(syn::Error::new(b.span(), "note: first one here"));
            Err(error)
        }
    }
}

pub(crate) fn tagged_impl_header(
    trait_name: &str,
    ident: &Ident,
    tag: Option<&Path>,
) -> TokenStream {
    let trait_name = Ident::new(trait_name, Span::call_site());
    match tag {
        Some(tag) => quote! { impl ::uniffi::#trait_name<#tag> for #ident },
        None => quote! { impl<T> ::uniffi::#trait_name<T> for #ident },
    }
}

mod kw {
    syn::custom_keyword!(tag);
}

#[derive(Default)]
pub(crate) struct CommonAttr {
    pub tag: Option<Path>,
}

impl UniffiAttribute for CommonAttr {
    fn parse_one(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::tag) {
            let _: kw::tag = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self {
                tag: Some(input.parse()?),
            })
        } else {
            Err(lookahead.error())
        }
    }

    fn merge(self, other: Self) -> syn::Result<Self> {
        Ok(Self {
            tag: either_attribute_arg(self.tag, other.tag)?,
        })
    }
}

// So CommonAttr can be used with `parse_macro_input!`
impl Parse for CommonAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_comma_separated(input)
    }
}
