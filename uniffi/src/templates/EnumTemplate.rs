{#
// For each enum declared in the IDL, we assume the caller as provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the IDL.
//
// The enum will be sent over the FFI as a u32, with values assigned according to the
// order of items *as declared in the IDL file*. This might be different to the order
// of items as declared in the rust code, but no harm will come from it.
#}
unsafe impl uniffi::support::ViaFfi for {{ e.name() }} {
    type Value = u32;
    fn into_ffi_value(&self) -> Self::Value {
        match self {
            // If the provided enum doesn't match the options defined in the IDL then
            // this match will fail to compile, with a type error to guide the way.
            {%- for value in e.values() %}
            {{ e.name() }}::{{ value }} => {{ loop.index }},
            {%- endfor %}
        }
    }
    fn try_from_ffi_value(v: Self::Value) -> anyhow::Result<Self> {
        Ok(match v {
            {%- for value in e.values() %}
            {{ loop.index }} => {{ e.name() }}::{{ value }},
            {%- endfor %}
            _ => anyhow::bail!("Invalid {{ e.name() }} enum value: {}", v),
        })
    }
}

impl uniffi::support::Lowerable for {{ e.name() }} {
    fn lower_into<B: uniffi::support::BufMut>(&self, buf: &mut B) {
        use uniffi::support::ViaFfi;
        buf.put_u32(self.into_ffi_value());
    }
}

impl uniffi::support::Liftable for {{ e.name() }} {
    fn try_lift_from<B: uniffi::support::Buf>(buf: &mut B) -> anyhow::Result<Self> {
        uniffi::support::check_remaining(buf, 4)?;
        <Self as uniffi::support::ViaFfi>::try_from_ffi_value(buf.get_u32())
    }
}
