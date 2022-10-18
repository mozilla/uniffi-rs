{#
// For each enum declared in the UDL, we assume the caller has provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the UDL.
//
// We define a unit-struct to implement the trait to sidestep Rust's orphan rule (ADR-0006). It's
// public so other crates can refer to it via an `[External='crate'] typedef`
#}

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for r#{{ e.name() }} {
    fn write(obj: Self, buf: &mut std::vec::Vec<u8>) {
        use uniffi::deps::bytes::BufMut;
        match obj {
            {%- for variant in e.variants() %}
            Self::r#{{ variant.name() }} { {% for field in variant.fields() %}r#{{ field.name() }}, {%- endfor %} } => {
                buf.put_i32({{ loop.index }});
                {% for field in variant.fields() -%}
                {{ field.type_()|ffi_converter }}::write(r#{{ field.name() }}, buf);
                {%- endfor %}
            },
            {%- endfor %}
        };
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<Self> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 4)?;
        Ok(match buf.get_i32() {
            {%- for variant in e.variants() %}
            {{ loop.index }} => Self::r#{{ variant.name() }}{% if variant.has_fields() %} {
                {% for field in variant.fields() %}
                r#{{ field.name() }}: {{ field.type_()|ffi_converter }}::try_read(buf)?,
                {%- endfor %}
            }{% endif %},
            {%- endfor %}
            v => uniffi::deps::anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }
}
