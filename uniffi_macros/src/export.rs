/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{visit_mut::VisitMut, Item, Type};

mod attributes;
mod callback_interface;
mod item;
mod scaffolding;
mod trait_interface;
mod utrait;

use self::{
    item::{ExportItem, ImplItem},
    scaffolding::{
        gen_constructor_scaffolding, gen_ffi_function, gen_fn_scaffolding, gen_method_scaffolding,
    },
};
use crate::util::{create_metadata_items, ident_to_string, mod_path};
pub use attributes::{AsyncRuntime, DefaultMap, ExportFnArgs};
pub use callback_interface::ffi_converter_callback_interface_impl;
pub use trait_interface::alter_trait;

// TODO(jplatte): Ensure no generics, …
// TODO(jplatte): Aggregate errors instead of short-circuiting, wherever possible

pub(crate) fn expand_export(
    mut item: Item,
    all_args: proc_macro::TokenStream,
    udl_mode: bool,
) -> syn::Result<TokenStream> {
    let mod_path = mod_path()?;
    // If the input is an `impl` block, rewrite any uses of the `Self` type
    // alias to the actual type, so we don't have to special-case it in the
    // metadata collection or scaffolding code generation (which generates
    // new functions outside of the `impl`).
    rewrite_self_type(&mut item);

    let metadata = ExportItem::new(item, all_args)?;

    match metadata {
        ExportItem::Function { sig, args } => {
            gen_fn_scaffolding(sig, args.async_runtime.as_ref(), udl_mode)
        }
        ExportItem::Impl {
            items,
            self_ident,
            args,
            trait_,
        } => {
            if let Some(rt) = &args.async_runtime {
                if items
                    .iter()
                    .all(|item| !matches!(item, ImplItem::Method(sig) if sig.is_async))
                {
                    return Err(syn::Error::new_spanned(
                        rt,
                        "no async methods in this impl block",
                    ));
                }
            }

            let item_tokens: TokenStream = items
                .into_iter()
                .map(|item| match item {
                    ImplItem::Constructor(sig) => {
                        let async_runtime =
                            sig.async_runtime.clone().or(args.async_runtime.clone());
                        gen_constructor_scaffolding(sig, async_runtime.as_ref(), udl_mode)
                    }
                    ImplItem::Method(sig) => {
                        let async_runtime =
                            sig.async_runtime.clone().or(args.async_runtime.clone());
                        gen_method_scaffolding(sig, async_runtime.as_ref(), udl_mode)
                    }
                })
                .collect::<syn::Result<_>>()?;
            let trait_impl_tokens = trait_.map(|t| {
                let object_name = ident_to_string(&self_ident);
                // TODO: parse trait path?
                let trait_name = ident_to_string(t.get_ident().expect("not a simple trait path"));
                let trait_path = "";
                let metadata_expr = quote! {
                    ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::OBJECT_TRAIT_IMPL)
                        .concat(::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE))
                        .concat_str(#mod_path)
                        .concat_str(#object_name)
                        .concat_str(#trait_name)
                        .concat_str(#trait_path)
                };
                create_metadata_items(
                    "object_trait_impl",
                    &format!("{object_name}_{trait_name}"),
                    metadata_expr,
                    None,
                )
            });
            Ok(quote_spanned! { self_ident.span() => #item_tokens #trait_impl_tokens })
        }
        ExportItem::Trait {
            items,
            self_ident,
            with_foreign,
            callback_interface_only: false,
            docstring,
            args,
        } => trait_interface::gen_trait_scaffolding(
            &mod_path,
            args,
            self_ident,
            items,
            udl_mode,
            with_foreign,
            docstring,
        ),
        ExportItem::Trait {
            items,
            self_ident,
            callback_interface_only: true,
            docstring,
            ..
        } => {
            let trait_name = ident_to_string(&self_ident);
            let trait_impl_ident = callback_interface::trait_impl_ident(&trait_name);
            let trait_impl = callback_interface::trait_impl(&mod_path, &self_ident, &items, false)
                .unwrap_or_else(|e| e.into_compile_error());
            let metadata_items = (!udl_mode).then(|| {
                let items =
                    callback_interface::metadata_items(&self_ident, &items, &mod_path, docstring)
                        .unwrap_or_else(|e| vec![e.into_compile_error()]);
                quote! { #(#items)* }
            });
            let ffi_converter_tokens =
                ffi_converter_callback_interface_impl(&self_ident, &trait_impl_ident);

            Ok(quote! {
                #trait_impl

                #ffi_converter_tokens

                #metadata_items
            })
        }
        ExportItem::Struct {
            self_ident,
            uniffi_traits,
            ..
        } => {
            let include_meta = !udl_mode;
            utrait::expand_uniffi_trait_export(self_ident, uniffi_traits, include_meta)
        }
    }
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

    impl VisitMut for RewriteSelfVisitor<'_> {
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

/// Alter the tokens wrapped with the `[uniffi::export]` if needed
pub fn alter_input(item: &Item) -> TokenStream {
    match item {
        Item::Trait(item_trait) => alter_trait(item_trait),
        _ => quote! { #item },
    }
}
