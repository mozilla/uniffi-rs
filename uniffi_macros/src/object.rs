use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Path};
use uniffi_meta::ObjectMetadata;

use crate::util::{
    assert_type_eq, create_metadata_static_var, tagged_impl_header, AttributeSliceExt, CommonAttr,
};

pub fn expand_object(input: DeriveInput, module_path: Vec<String>) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let attr = input.attrs.parse_uniffi_attributes::<CommonAttr>()?;
    let name = ident.to_string();
    let metadata = ObjectMetadata { module_path, name };
    let free_fn_ident = Ident::new(&metadata.free_ffi_symbol_name(), Span::call_site());
    let meta_static_var = create_metadata_static_var(ident, metadata.into());
    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });
    let interface_impl = interface_impl(ident, attr.tag.as_ref());

    Ok(quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #free_fn_ident(
            ptr: *const ::std::ffi::c_void,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            uniffi::call_with_output(call_status, || {
                assert!(!ptr.is_null());
                let ptr = ptr.cast::<#ident>();
                unsafe {
                    ::std::sync::Arc::decrement_strong_count(ptr);
                }
            });
        }

        #interface_impl
        #meta_static_var
        #type_assertion
    })
}

pub(crate) fn expand_ffi_converter_interface(attr: CommonAttr, input: DeriveInput) -> TokenStream {
    interface_impl(&input.ident, attr.tag.as_ref())
}

pub(crate) fn interface_impl(ident: &Ident, tag: Option<&Path>) -> TokenStream {
    let impl_spec = tagged_impl_header("Interface", ident, tag);
    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        #impl_spec { }
    }
}
