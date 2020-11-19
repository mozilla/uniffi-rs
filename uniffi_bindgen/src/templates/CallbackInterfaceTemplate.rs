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
//    - a `Send` `impl` so `Object`s can store callbacks.
#}
{% let trait_name = obj.name() -%}
{% let trait_impl = format!("{}Proxy", trait_name) -%}
{% let foreign_callback_internals = format!("foreign_callback_{}_internals", trait_name)|upper -%}

// Register a foreign callback for getting across the FFI.
static {{ foreign_callback_internals }}: uniffi::ForeignCallbackInternals = uniffi::ForeignCallbackInternals::new();

#[no_mangle]
pub extern "C" fn {{ obj.ffi_init_callback().name() }}(callback: uniffi::ForeignCallback) {
    {{ foreign_callback_internals }}.set_callback(callback);
}

// Make an implementation which will shell out to the foreign language.
#[derive(Debug)]
struct {{ trait_impl }} {
  handle: u64
}

// We need Send so we can stash the callbacks in objects we're sharing with foreign languages
// i.e. in the handle map.
// Prepared for our target trait to declare:
//  `trait {{ trait_name }}: Send + std::fmt::Debug`
unsafe impl Send for {{ trait_impl }} {}

impl Drop for {{ trait_impl }} {
    fn drop(&mut self) {
        let callback = {{ foreign_callback_internals }}.get_callback().unwrap();
        unsafe { callback(self.handle, uniffi::IDX_CALLBACK_FREE, Default::default()) };
    }
}

impl {{ trait_name }} for {{ trait_impl }} {
    {%- for meth in obj.methods() %}

    {#- Method declaration #}
    fn {{ meth.name() -}}
    ({% call rs::arg_list_decl_with_prefix("&self", meth) %})
    {%- match meth.return_type() %}
    {%- when Some with (return_type) %} -> {{ return_type|type_rs }}
    {% else -%}
    {%- endmatch -%} { 
    {#- Method body #}
        uniffi::deps::log::debug!("{{ obj.name() }}.{{ meth.name() }}");

    {#- Packing args into a RustBuffer #}
        {% if meth.arguments().len() == 0 -%}
        let args_buf = Vec::new();
        {% else -%}
        let mut args_buf = Vec::new();
        {% endif -%}
        {% for arg in meth.arguments() -%}
            {{ arg.name()|write_rs("&mut args_buf", arg.type_()) -}};
        {% endfor -%}
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);

    {#- Calling into foreign code. #}
        let callback = {{ foreign_callback_internals }}.get_callback().unwrap();
        let ret_rbuf = unsafe { callback(self.handle, {{ loop.index }}, args_rbuf) };

    {#- Unpacking the RustBuffer to return to Rust #}
        {% match meth.return_type() -%}
        {% when Some with (return_type) -%}
        let vec = ret_rbuf.destroy_into_vec();
        let mut ret_buf = vec.as_slice();
        let rval = {{ "&mut ret_buf"|read_rs(return_type) }};
        rval
        {%- else -%}
        uniffi::RustBuffer::destroy(ret_rbuf);
        {%- endmatch %}
    }
    {%- endfor %}
}

unsafe impl uniffi::ViaFfi for {{ trait_impl }} {
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
    fn lower(self) -> Self::FfiType {
        self.handle
    }

    fn write<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u64(self.handle);
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        Ok(Self { handle: v })
    }

    fn try_read<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::check_remaining(buf, 8)?;
        <Self as uniffi::ViaFfi>::try_lift(buf.get_u64())
    }
}