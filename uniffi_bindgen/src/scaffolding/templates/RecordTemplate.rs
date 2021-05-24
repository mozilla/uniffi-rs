{#
// For each record declared in the UDL, we assume the caller has provided a corresponding
// rust `struct` with the declared fields. We provide the traits for sending it across the FFI.
// If the caller's struct does not match the shape and types declared in the UDL then the rust
// compiler will complain with a type error.
#}
#[doc(hidden)]
unsafe impl uniffi::ViaFfi for {{ rec.name() }} {
    type FfiType = uniffi::RustBuffer;

    fn lower(self) -> Self::FfiType {
        uniffi::lower_into_buffer(self)
    }

    fn try_lift(v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::try_lift_from_buffer(v)
    }

    fn write(&self, buf: &mut Vec<u8>) {
        // If the provided struct doesn't match the fields declared in the UDL, then
        // the generated code here will fail to compile with somewhat helpful error.
        {%- for field in rec.fields() %}
        uniffi::ViaFfi::write(&self.{{ field.name() }}, buf);
        {%- endfor %}
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::deps::anyhow::Result<Self> {
        Ok(Self {
            {%- for field in rec.fields() %}
                {{ field.name() }}: <{{ field.type_()|type_rs }} as uniffi::ViaFfi>::try_read(buf)?,
            {%- endfor %}
        })
    }
}
