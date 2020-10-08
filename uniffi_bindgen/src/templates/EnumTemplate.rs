{#
// For each enum declared in the UDL, we assume the caller as provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the UDL.
//
// The enum will be sent over the FFI as a u32, with values assigned according to the
// order of items *as declared in the UDL file*. This might be different to the order
// of items as declared in the rust code, but no harm will come from it.
#}
unsafe impl uniffi::ViaFfi for {{ e.name() }} {
    type FfiType = u32;

    fn lower(self) -> Self::FfiType {
        match self {
            // If the provided enum doesn't match the options defined in the UDL then
            // this match will fail to compile, with a type error to guide the way.
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant }} => {{ loop.index }},
            {%- endfor %}
        }
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        Ok(match v {
            {%- for variant in e.variants() %}
            {{ loop.index }} => {{ e.name() }}::{{ variant }},
            {%- endfor %}
            _ => uniffi::deps::anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }

    fn write<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u32(match self {
            {%- for variant in e.variants() %}
            {{ e.name() }}::{{ variant }} => {{ loop.index }},
            {%- endfor %}
        });
    }

    fn try_read<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::check_remaining(buf, 4)?;
        <Self as uniffi::ViaFfi>::try_lift(buf.get_u32())
    }
}
