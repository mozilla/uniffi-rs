use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Path};
use uniffi_meta::free_fn_symbol_name;

use crate::util::{
    create_metadata_items, ident_to_string, tagged_impl_header, AttributeSliceExt, CommonAttr,
};

pub fn expand_object(input: DeriveInput, module_path: String) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attr_args::<CommonAttr>()?;
    let name = ident_to_string(ident);
    let free_fn_ident = Ident::new(&free_fn_symbol_name(&module_path, &name), Span::call_site());
    let meta_static_var = interface_meta_static_var(ident, &module_path)?;
    let interface_impl = interface_impl(ident, attr.tag.as_ref());

    Ok(quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #free_fn_ident(
            ptr: *const ::std::ffi::c_void,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::rust_call(call_status, || {
                assert!(!ptr.is_null());
                let ptr = ptr.cast::<#ident>();
                unsafe {
                    ::std::sync::Arc::decrement_strong_count(ptr);
                }
                Ok(())
            });
        }

        #interface_impl
        #meta_static_var
    })
}

pub(crate) fn expand_ffi_converter_interface(attr: CommonAttr, input: DeriveInput) -> TokenStream {
    interface_impl(&input.ident, attr.tag.as_ref())
}

pub(crate) fn interface_impl(ident: &Ident, tag: Option<&Path>) -> TokenStream {
    let name = ident_to_string(ident);
    let impl_spec = tagged_impl_header("Interface", ident, tag);
    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        #impl_spec {
            const NAME: &'static str = #name;
        }
    }
}

pub(crate) fn interface_meta_static_var(
    ident: &Ident,
    module_path: &str,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    Ok(create_metadata_items(
        "interface",
        &name,
        quote! {
                ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::INTERFACE)
                    .concat_str(#module_path)
                    .concat_str(#name)
        },
        None,
    ))
}
