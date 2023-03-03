{#
// For each Callback Interface definition, we assume that there is a corresponding trait defined in Rust client code.
// If the UDL callback interface and Rust trait's methods don't match, the Rust compiler will complain.
// We generate:
//  * an init function to accept that `ForeignCallback` from the foreign language, and stores it.
//  * a holder for a `ForeignCallback`, of type `uniffi::ForeignCallbackInternals`.
//  * a proxy `struct` which implements the `trait` that the Callback Interface corresponds to. This
//    is the object that client code interacts with.
//    - for each method, arguments will be packed into a `RustBuffer` and sent over the `ForeignCallback` to be
//      unpacked and called. The return value is packed into another `RustBuffer` and sent back to Rust.
//    - a `Drop` `impl`, which tells the foreign language to forget about the real callback object.
#}
{% let trait_name = cbi.name() -%}
{% let trait_impl = format!("UniFFICallbackHandler{}", trait_name) %}
{% let foreign_callback_internals = format!("foreign_callback_{}_internals", trait_name)|upper -%}

// Register a foreign callback for getting across the FFI.
#[doc(hidden)]
static {{ foreign_callback_internals }}: uniffi::ForeignCallbackInternals = uniffi::ForeignCallbackInternals::new();

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ cbi.ffi_init_callback().name() }}(callback: uniffi::ForeignCallback, _: &mut uniffi::RustCallStatus) {
    {{ foreign_callback_internals }}.set_callback(callback);
    // The call status should be initialized to CALL_SUCCESS, so no need to modify it.
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ cbi.ffi_init_callback2().name() }}(callback: uniffi::ForeignCallback2, _: &mut uniffi::RustCallStatus) {
    {{ foreign_callback_internals }}.set_callback2(callback);
    // The call status should be initialized to CALL_SUCCESS, so no need to modify it.
}

// Make an implementation which will shell out to the foreign language.
#[doc(hidden)]
#[derive(Debug)]
struct {{ trait_impl }} {
  handle: u64
}

impl Drop for {{ trait_impl }} {
    fn drop(&mut self) {
        {{ foreign_callback_internals }}.invoke_callback_void_return(
            self.handle, uniffi::IDX_CALLBACK_FREE, Default::default()
        )
    }
}

uniffi::deps::static_assertions::assert_impl_all!({{ trait_impl }}: Send);

impl r#{{ trait_name }} for {{ trait_impl }} {
    {%- for meth in cbi.methods() %}

    {#- Method declaration #}
    fn r#{{ meth.name() -}}
    ({% call rs::arg_list_decl_with_prefix("&self", meth) %})
    {%- match (meth.return_type(), meth.throws_type()) %}
    {%- when (Some(return_type), None) %} -> {{ return_type.borrow()|type_rs }}
    {%- when (Some(return_type), Some(err)) %} -> ::std::result::Result<{{ return_type.borrow()|type_rs }}, {{ err|type_rs }}>
    {%- when (None, Some(err)) %} -> ::std::result::Result<(), {{ err|type_rs }}>
    {% else -%}
    {%- endmatch -%} {
    {#- Method body #}

    {#- Packing args into a RustBuffer #}
        {% if meth.arguments().len() == 0 -%}
        let args_buf = Vec::new();
        {% else -%}
        let mut args_buf = Vec::new();
        {% endif -%}
        {%- for arg in meth.arguments() %}
        {{ arg.type_().borrow()|ffi_converter }}::write(r#{{ arg.name() }}, &mut args_buf);
        {%- endfor -%}
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);

    {#- Calling into foreign code. #}
        {%- match (meth.return_type(), meth.throws_type()) %}
        {%- when (Some(return_type), None) %}
        {{ foreign_callback_internals }}.invoke_callback::<{{ return_type|type_rs }}, crate::UniFfiTag>(self.handle, {{ loop.index }}, args_rbuf)

        {%- when (Some(return_type), Some(error_type)) %}
        {{ foreign_callback_internals }}.invoke_callback_with_error::<{{ return_type|type_rs }}, {{ error_type|type_rs }}, crate::UniFfiTag>(self.handle, {{ loop.index }}, args_rbuf)

        {%- when (None, Some(error_type)) %}
        {{ foreign_callback_internals }}.invoke_callback_with_error::<(), {{ error_type|type_rs }}, crate::UniFfiTag>(self.handle, {{ loop.index }}, args_rbuf)

        {%- when (None, None) %}
        {{ foreign_callback_internals }}.invoke_callback_void_return(self.handle, {{ loop.index }}, args_rbuf)

        {%- endmatch %}
    }
    {%- endfor %}
}

unsafe impl ::uniffi::FfiConverter<crate::UniFfiTag> for Box<dyn r#{{ trait_name }}> {
    type FfiType = u64;

    // Lower and write are tricky to implement because we have a dyn trait as our type.  There's
    // probably a way to, but this carries lots of thread safety risks, down to impedance
    // mismatches between Rust and foreign languages, and our uncertainty around implementations of
    // concurrent handlemaps.
    //
    // The use case for them is also quite exotic: it's passing a foreign callback back to the foreign
    // language.
    //
    // Until we have some certainty, and use cases, we shouldn't use them.
    fn lower(_obj: Box<dyn r#{{ trait_name }}>) -> Self::FfiType {
        panic!("Lowering CallbackInterface not supported")
    }

    fn write(_obj: Box<dyn r#{{ trait_name }}>, _buf: &mut std::vec::Vec<u8>) {
        panic!("Writing CallbackInterface not supported")
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Box<dyn r#{{ trait_name }}>> {
        Ok(Box::new({{ trait_impl }} { handle: v }))
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<Box<dyn r#{{ trait_name }}>> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 8)?;
        Self::try_lift(buf.get_u64())
    }
}
