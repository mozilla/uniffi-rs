{#
// For each enum declared in the UDL, we assume the caller has provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the UDL.
#}
#[doc(hidden)]
unsafe impl uniffi::ViaFfi for {{ e.name() }} {
    type FfiType = uniffi::RustBuffer;

    fn lower(self) -> Self::FfiType {
        uniffi::lower_into_buffer(self)
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::try_lift_from_buffer(v)
    }

    fn write<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        match self {
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant.name() }} { {% for field in variant.fields() %}{{ field.name() }}, {%- endfor %} } => {
                buf.put_i32({{ loop.index }});
                {% for field in variant.fields() -%}
                <{{ field.type_()|type_rs }} as uniffi::ViaFfi>::write({{ field.name() }}, buf);
                {%- endfor %}
            },
            {%- endfor %}
        };
    }

    fn try_read<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::check_remaining(buf, 4)?;
        Ok(match buf.get_i32() {
            {%- for variant in e.variants() %}
            {{ loop.index }} => {{ e.name() }}::{{ variant.name() }}{% if variant.has_fields() %} {
                {% for field in variant.fields() %}
                {{ field.name() }}: <{{ field.type_()|type_rs }} as uniffi::ViaFfi>::try_read(buf)?,
                {%- endfor %}
            }{% endif %},
            {%- endfor %}
            v @ _ => uniffi::deps::anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }
}
