use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

use crate::{
    ffiops,
    util::{create_metadata_items, extract_docstring, ident_to_string, mod_path},
    DeriveOptions,
};
use uniffi_meta::ObjectImpl;

/// Stores parsed data from the Derive Input for the struct/enum.
struct ObjectItem {
    ident: Ident,
    docstring: String,
}

impl ObjectItem {
    fn new(input: DeriveInput) -> syn::Result<Self> {
        Ok(Self {
            ident: input.ident,
            docstring: extract_docstring(&input.attrs)?,
        })
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }

    fn name(&self) -> String {
        ident_to_string(&self.ident)
    }

    fn docstring(&self) -> &str {
        self.docstring.as_str()
    }
}

pub fn expand_object(input: DeriveInput, options: DeriveOptions) -> syn::Result<TokenStream> {
    let module_path = mod_path()?;
    let object = ObjectItem::new(input)?;
    let name = object.name();
    let ident = object.ident();
    let clone_fn_ident = Ident::new(
        &uniffi_meta::clone_fn_symbol_name(&module_path, &name),
        Span::call_site(),
    );
    let free_fn_ident = Ident::new(
        &uniffi_meta::free_fn_symbol_name(&module_path, &name),
        Span::call_site(),
    );
    let meta_static_var = options.generate_metadata.then(|| {
        interface_meta_static_var(
            object.ident(),
            ObjectImpl::Struct,
            &module_path,
            object.docstring(),
        )
        .unwrap_or_else(syn::Error::into_compile_error)
    });
    let interface_impl = interface_impl(&object, &options);

    Ok(quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #clone_fn_ident(
            ptr: *const ::std::ffi::c_void,
            call_status: &mut ::uniffi::RustCallStatus
        ) -> *const ::std::ffi::c_void {
            ::uniffi::rust_call(call_status, || {
                unsafe { ::std::sync::Arc::increment_strong_count(ptr) };
                ::std::result::Result::Ok(ptr)
            })
        }

        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn #free_fn_ident(
            ptr: *const ::std::ffi::c_void,
            call_status: &mut ::uniffi::RustCallStatus
        ) {
            ::uniffi::rust_call(call_status, || {
                assert!(!ptr.is_null());
                let ptr = ptr.cast::<#ident>();
                unsafe {
                    ::std::sync::Arc::decrement_strong_count(ptr);
                }
                ::std::result::Result::Ok(())
            });
        }

        #interface_impl
        #meta_static_var
    })
}

fn interface_impl(object: &ObjectItem, options: &DeriveOptions) -> TokenStream {
    let name = object.name();
    let ident = object.ident();
    let impl_spec = options.ffi_impl_header("FfiConverterArc", ident);
    let lower_return_impl_spec = options.ffi_impl_header("LowerReturn", ident);
    let lower_error_impl_spec = options.ffi_impl_header("LowerError", ident);
    let type_id_impl_spec = options.ffi_impl_header("TypeId", ident);
    let lift_ref_impl_spec = options.ffi_impl_header("LiftRef", ident);
    let mod_path = match mod_path() {
        Ok(p) => p,
        Err(e) => return e.into_compile_error(),
    };
    let arc_self_type = quote! { ::std::sync::Arc<Self> };
    let lower_arc = ffiops::lower(&arc_self_type);
    let type_id_meta_arc = ffiops::type_id_meta(&arc_self_type);
    let try_lift_arc = ffiops::try_lift(&arc_self_type);
    let lower_return_type_arc = ffiops::lower_return_type(&arc_self_type);
    let lower_return_arc = ffiops::lower_return(&arc_self_type);
    let lower_error_arc = ffiops::lower_error(&arc_self_type);

    quote! {
        // All Object structs must be `Sync + Send`. The generated scaffolding will fail to compile
        // if they are not, but unfortunately it fails with an unactionably obscure error message.
        // By asserting the requirement explicitly, we help Rust produce a more scrutable error message
        // and thus help the user debug why the requirement isn't being met.
        ::uniffi::deps::static_assertions::assert_impl_all!(
            #ident: ::core::marker::Sync, ::core::marker::Send
        );

        #[doc(hidden)]
        #[automatically_derived]
        /// Support for passing reference-counted shared objects via the FFI.
        ///
        /// To avoid dealing with complex lifetime semantics over the FFI, any data passed
        /// by reference must be encapsulated in an `Arc`, and must be safe to share
        /// across threads.
        unsafe #impl_spec {
            // Don't use a pointer to <T> as that requires a `pub <T>`
            type FfiType = *const ::std::os::raw::c_void;

            /// When lowering, we have an owned `Arc` and we transfer that ownership
            /// to the foreign-language code, "leaking" it out of Rust's ownership system
            /// as a raw pointer. This works safely because we have unique ownership of `self`.
            /// The foreign-language code is responsible for freeing this by calling the
            /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
            ///
            /// Safety: when freeing the resulting pointer, the foreign-language code must
            /// call the destructor function specific to the type `T`. Calling the destructor
            /// function for other types may lead to undefined behaviour.
            fn lower(obj: ::std::sync::Arc<Self>) -> Self::FfiType {
                ::std::sync::Arc::into_raw(obj) as Self::FfiType
            }

            /// When lifting, we receive an owned `Arc` that the foreign language code cloned.
            fn try_lift(v: Self::FfiType) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                let v = v as *const #ident;
                ::std::result::Result::Ok(unsafe { ::std::sync::Arc::<Self>::from_raw(v) })
            }

            /// When writing as a field of a complex structure, make a clone and transfer ownership
            /// of it to the foreign-language code by writing its pointer into the buffer.
            /// The foreign-language code is responsible for freeing this by calling the
            /// `ffi_object_free` FFI function provided by the corresponding UniFFI type.
            ///
            /// Safety: when freeing the resulting pointer, the foreign-language code must
            /// call the destructor function specific to the type `T`. Calling the destructor
            /// function for other types may lead to undefined behaviour.
            fn write(obj: ::std::sync::Arc<Self>, buf: &mut ::std::vec::Vec<u8>) {
                ::uniffi::deps::static_assertions::const_assert!(::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8);
                ::uniffi::deps::bytes::BufMut::put_u64(buf, #lower_arc(obj) as ::std::primitive::u64);
            }

            /// When reading as a field of a complex structure, we receive a "borrow" of the `Arc`
            /// that is owned by the foreign-language code, and make a clone for our own use.
            ///
            /// Safety: the buffer must contain a pointer previously obtained by calling
            /// the `lower()` or `write()` method of this impl.
            fn try_read(buf: &mut &[u8]) -> ::uniffi::Result<::std::sync::Arc<Self>> {
                ::uniffi::deps::static_assertions::const_assert!(::std::mem::size_of::<*const ::std::ffi::c_void>() <= 8);
                ::uniffi::check_remaining(buf, 8)?;
                #try_lift_arc(::uniffi::deps::bytes::Buf::get_u64(buf) as Self::FfiType)
            }

            const TYPE_ID_META: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::TYPE_INTERFACE)
                .concat_str(#mod_path)
                .concat_str(#name);
        }

        unsafe #lower_return_impl_spec {
            type ReturnType = #lower_return_type_arc;

            fn lower_return(obj: Self) -> ::std::result::Result<Self::ReturnType, ::uniffi::RustCallError> {
                #lower_return_arc(::std::sync::Arc::new(obj))
            }
        }

        unsafe #lower_error_impl_spec {
            fn lower_error(obj: Self) -> ::uniffi::RustBuffer {
                #lower_error_arc(::std::sync::Arc::new(obj))
            }
        }

        unsafe #lift_ref_impl_spec {
            type LiftType = ::std::sync::Arc<Self>;
        }

        #type_id_impl_spec {
            const TYPE_ID_META: ::uniffi::MetadataBuffer = #type_id_meta_arc;
        }
    }
}

pub(crate) fn interface_meta_static_var(
    ident: &Ident,
    imp: ObjectImpl,
    module_path: &str,
    docstring: &str,
) -> syn::Result<TokenStream> {
    let name = ident_to_string(ident);
    let code = match imp {
        ObjectImpl::Struct => quote! { ::uniffi::metadata::codes::INTERFACE },
        ObjectImpl::Trait => quote! { ::uniffi::metadata::codes::TRAIT_INTERFACE },
        ObjectImpl::CallbackTrait => quote! { ::uniffi::metadata::codes::CALLBACK_TRAIT_INTERFACE },
    };

    Ok(create_metadata_items(
        "interface",
        &name,
        quote! {
            ::uniffi::MetadataBuffer::from_code(#code)
                .concat_str(#module_path)
                .concat_str(#name)
                .concat_long_str(#docstring)
        },
        None,
    ))
}
