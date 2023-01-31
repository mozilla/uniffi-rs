use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{AttributeArgs, DeriveInput};
use uniffi_meta::ObjectMetadata;

use crate::util::{assert_type_eq, create_metadata_static_var, FfiConverterTagHandler};

pub fn expand_object(input: DeriveInput, module_path: Vec<String>) -> TokenStream {
    let ident = &input.ident;
    let name = ident.to_string();
    let metadata = ObjectMetadata { module_path, name };
    let free_fn_ident = Ident::new(&metadata.free_ffi_symbol_name(), Span::call_site());
    let meta_static_var = create_metadata_static_var(ident, metadata.into());
    let type_assertion = assert_type_eq(ident, quote! { crate::uniffi_types::#ident });
    let interface_impl = interface_impl(ident, FfiConverterTagHandler::generic_impl());

    quote! {
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
    }
}

pub fn expand_ffi_converter_interface(attrs: AttributeArgs, input: DeriveInput) -> TokenStream {
    let tag_handler = match FfiConverterTagHandler::try_from(attrs) {
        Ok(tag_handler) => tag_handler,
        Err(e) => return e.into_compile_error(),
    };
    interface_impl(&input.ident, tag_handler)
}

pub(crate) fn interface_impl(ident: &Ident, tag_handler: FfiConverterTagHandler) -> TokenStream {
    let (impl_spec, _) = tag_handler.into_impl_and_tag_path("Interface", ident);
    quote! {
        #[doc(hidden)]
        #[automatically_derived]
        #impl_spec { }
    }
}
