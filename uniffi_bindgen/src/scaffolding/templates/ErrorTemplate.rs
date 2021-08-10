{#
// For each error declared in the UDL, we assume the caller has provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the UDL.
//
// We define a unit-struct to implement the trait to sidestep Rust's orphan rule (ADR-0006).
#}

struct {{ e.type_()|ffi_converter_name }};

#[doc(hidden)]
impl uniffi::RustBufferFfiConverter for {{ e.type_()|ffi_converter_name }} {
    type RustType = {{ e.name() }};

    fn write(obj: {{ e.name() }}, buf: &mut std::vec::Vec<u8>) {
        use uniffi::deps::bytes::BufMut;
        match obj {
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant.name() }}{% if variant.has_fields() %} { {% for field in variant.fields() %}{{ field.name() }}, {%- endfor %} }{% else %}{..}{% endif %} => {
                buf.put_i32({{ loop.index }});
                {% for field in variant.fields() -%}
                {{ field.type_()|ffi_converter }}::write({{ field.name() }}, buf);
                {%- endfor %}
            },
            {%- endfor %}
        };
    }

    {% if e.is_flat() %}
    // If a variant doesn't have fields defined in the UDL, it's currently still possible that
    // the Rust enum has fields and they're just not listed.  Let's just punt on implementing
    // try_read() to avoid that case.  It should be no issue since passing back Errors into the
    // rust code isn't supported.
    fn try_read(_buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<{{ e.name() }}> {
        panic!("try_read not supported for fieldless errors");
    }
    {% else %}
    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<{{ e.name() }}> {
        use uniffi::deps::bytes::Buf;
        uniffi::check_remaining(buf, 4)?;
        Ok(match buf.get_i32() {
            {%- for variant in e.variants() %}
            {{ loop.index }} => {{ e.name() }}::{{ variant.name() }}{% if variant.has_fields() %} {
                {% for field in variant.fields() %}
                {{ field.name() }}: {{ field.type_()|ffi_converter }}::try_read(buf)?,
                {%- endfor %}
            }{% endif %},
            {%- endfor %}
            v => uniffi::deps::anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }
    {% endif %}
}

impl uniffi::FfiError for {{ e.type_()|ffi_converter_name }} { }
