{#
// For each enum declared in the IDL, we assume the caller as provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the IDL.
//
// The enum will be sent over the FFI as a u32, with values assigned according to the
// order of items *as declared in the IDL file*. This might be different to the order
// of items as declared in the rust code, but no harm will come from it.
#}
unsafe impl uniffi::ViaFfi for {{ e.name() }} {
    type Value = u32;
    fn into_ffi_value(self) -> Self::Value {
        match self {
            // If the provided enum doesn't match the options defined in the IDL then
            // this match will fail to compile, with a type error to guide the way.
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant }} => {{ loop.index }},
            {%- endfor %}
        }
    }
    fn try_from_ffi_value(v: Self::Value) -> uniffi::deps::anyhow::Result<Self> {
        Ok(match v {
            {%- for variant in e.variants() %}
            {{ loop.index }} => {{ e.name() }}::{{ variant }},
            {%- endfor %}
            _ => uniffi::deps::anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }
}

impl uniffi::Lowerable for {{ e.name() }} {
    fn lower_into<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u32(match self {
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant }} => {{ loop.index }},
            {%- endfor %}
        });
    }
}

impl uniffi::Liftable for {{ e.name() }} {
    fn try_lift_from<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::check_remaining(buf, 4)?;
        <Self as uniffi::ViaFfi>::try_from_ffi_value(buf.get_u32())
    }
}
