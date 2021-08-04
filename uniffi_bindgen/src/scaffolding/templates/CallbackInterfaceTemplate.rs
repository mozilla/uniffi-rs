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
{% let trait_impl = cbi.type_().viaffi_impl_name() -%}
{% let foreign_callback_internals = format!("foreign_callback_{}_internals", trait_name)|upper -%}

// Register a foreign callback for getting across the FFI.
#[doc(hidden)]
static {{ foreign_callback_internals }}: uniffi::ForeignCallbackInternals = uniffi::ForeignCallbackInternals::new();

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ cbi.ffi_init_callback().name() }}(callback: uniffi::ForeignCallback) {
    {{ foreign_callback_internals }}.set_callback(callback);
}

// Make an implementation which will shell out to the foreign language.
#[doc(hidden)]
#[derive(Debug)]
struct {{ trait_impl }} {
  handle: u64
}

impl Drop for {{ trait_impl }} {
    fn drop(&mut self) {
        let callback = {{ foreign_callback_internals }}.get_callback().unwrap();
        unsafe { callback(self.handle, uniffi::IDX_CALLBACK_FREE, Default::default()) };
    }
}

uniffi::deps::static_assertions::assert_impl_all!({{ trait_impl }}: Send);

impl {{ trait_name }} for {{ trait_impl }} {
    {%- for meth in cbi.methods() %}

    {#- Method declaration #}
    fn {{ meth.name() -}}
    ({% call rs::arg_list_decl_with_prefix("&self", meth) %})
    {%- match meth.return_type() %}
    {%- when Some with (return_type) %} -> {{ return_type|type_rs }}
    {% else -%}
    {%- endmatch -%} { 
    {#- Method body #}
        uniffi::deps::log::debug!("{{ cbi.name() }}.{{ meth.name() }}");

    {#- Packing args into a RustBuffer #}
        {% if meth.arguments().len() == 0 -%}
        let args_buf = Vec::new();
        {% else -%}
        let mut args_buf = Vec::new();
        {% endif -%}
        {%- for arg in meth.arguments() %}
        {{ arg.type_()|as_viaffi }}::write({{ arg.name() }}, &mut args_buf);
        {%- endfor -%}
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);

    {#- Calling into foreign code. #}
        let callback = {{ foreign_callback_internals }}.get_callback().unwrap();
        let ret_rbuf = unsafe { callback(self.handle, {{ loop.index }}, args_rbuf) };

    {#- Unpacking the RustBuffer to return to Rust #}
        {% match meth.return_type() -%}
        {% when Some with (return_type) -%}
        let vec = ret_rbuf.destroy_into_vec();
        let mut ret_buf = vec.as_slice();
        {{ return_type|as_viaffi }}::try_read(&mut ret_buf).unwrap()
        {%- else -%}
        uniffi::RustBuffer::destroy(ret_rbuf);
        {%- endmatch %}
    }
    {%- endfor %}
}

unsafe impl uniffi::ViaFfi for {{ trait_impl }} {
    // CallbackInterface instances get wrapped by Box<>, which allows the rust code to accept them
    // as parameters using Box<dyn CallbackInterfaceTrait>
    type RustType = Box<Self>;
    type FfiType = u64;
    
    // Lower and write are trivially implemented, but carry lots of thread safety risks, down to
    // impedence mismatches between Rust and foreign languages, and our uncertainty around implementations
    // of concurrent handlemaps.
    //
    // The use case for them is also quite exotic: it's passing a foreign callback back to the foreign
    // language.
    //
    // Until we have some certainty, and use cases, we shouldn't use them. 
    // 
    // They are implemented here for runtime use, but at scaffolding.rs will bail instead of generating
    // the code to call these methods.
    fn lower(obj: Self::RustType) -> Self::FfiType {
        obj.handle
    }

    fn write(obj: Self::RustType, buf: &mut Vec<u8>) {
        use uniffi::deps::bytes::BufMut;
        buf.put_u64(obj.handle);
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Box<Self>> {
        Ok(Box::new(Self { handle: v }))
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<Box<Self>> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 8)?;
        <Self as uniffi::ViaFfi>::try_lift(buf.get_u64())
    }
}
