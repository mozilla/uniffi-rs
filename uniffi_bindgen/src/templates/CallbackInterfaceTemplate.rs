{#
// For each callback interface in the UDL, we assume the caller has provided a corresponding
// Rust `trait` with the declared methods. We provide the traits for sending it across the FFI.
// If the caller's trait does not match the shape and types declared in the UDL then the rust
// compiler will complain with a type error.
// 
// The generated proxy will implement `Drop`, `Send` and `Debug`.
#}
{% let trait_name = obj.name() -%}
{% let trait_impl = format!("{}Proxy", trait_name) -%}
{% let foreign_callback_holder = format!("foreign_callback_{}_holder", trait_name)|upper -%}

// Register a foreign callback for getting across the FFI.
static {{ foreign_callback_holder }}: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn {{ obj.ffi_init_callback().name() }}(callback: uniffi::ForeignCallback) {
    uniffi::set_foreign_callback(&{{ foreign_callback_holder }}, callback);
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
        let callback = uniffi::get_foreign_callback(&{{ foreign_callback_holder }}).unwrap();

    {#- Packing args into a RustBuffer #}
        {% if meth.arguments().len() == 0 -%}
        let args_rbuf = uniffi::RustBuffer::new();
        {% else -%}
        let mut args_buf = Vec::new();
        {% for arg in meth.arguments() -%}
            {{ arg.name()|write_rs("&mut args_buf", arg.type_()) -}};
        {% endfor -%}
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);
        {% endif -%}

    {#- Calling into foreign code. #}
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

    fn lower(self) -> Self::FfiType {
        self.handle
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        Ok(Self { handle: v })
    }

    fn write<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u64(self.handle);
    }

    fn try_read<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::check_remaining(buf, 8)?;
        <Self as uniffi::ViaFfi>::try_lift(buf.get_u64())
    }
}

impl Drop for {{ trait_impl }} {
    fn drop(&mut self) {
        let callback = uniffi::get_foreign_callback(&{{ foreign_callback_holder }}).unwrap();
        unsafe { callback(self.handle, 0, Default::default()) };
    }
}