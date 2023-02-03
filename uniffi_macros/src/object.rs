use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Path};
use uniffi_meta::ObjectMetadata;

use crate::util::{
    create_metadata_static_var, tagged_impl_header, type_name, AttributeSliceExt, CommonAttr,
};

pub fn expand_object(input: DeriveInput, module_path: String) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attributes::<CommonAttr>()?;
    let name = type_name(ident);
    let metadata = ObjectMetadata { module_path, name };
    let free_fn_ident = Ident::new(&metadata.free_ffi_symbol_name(), Span::call_site());
    let meta_static_var = interface_meta_static_var(ident)?;
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
    let name = type_name(ident);
    let impl_spec = tagged_impl_header("Interface", ident, tag);
    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        #impl_spec {
            const NAME: &'static str = #name;
        }
    }
}

pub(crate) fn interface_meta_static_var(ident: &Ident) -> syn::Result<TokenStream> {
    let name = type_name(ident);
    Ok(create_metadata_static_var(
        "INTERFACE",
        &name,
        quote! {
                ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::INTERFACE)
                    .concat_str(module_path!())
                    .concat_str(#name)
        },
    ))
}
