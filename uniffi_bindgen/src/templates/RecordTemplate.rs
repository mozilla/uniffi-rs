{#
// For each record declared in the IDL, we assume the caller has provided a corresponding
// rust `struct` with the declared fields. We provide the traits for sending it across the FFI.
// If the caller's struct does not match the shape and types declared in the IDL then the rust
// compiler will complain with a type error.
#}
impl uniffi::Lowerable for {{ rec.name() }} {
    fn lower_into<B: uniffi::deps::bytes::BufMut>(&self, buf: &mut B) {
        // If the provided struct doesn't match the fields declared in the IDL, then
        // the generated code here will fail to compile with somewhat helpful error.
        {%- for field in rec.fields() %}
        uniffi::Lowerable::lower_into(&self.{{ field.name() }}, buf);
        {%- endfor %}
    }
}

impl uniffi::Liftable for {{ rec.name() }} {
    fn try_lift_from<B: uniffi::deps::bytes::Buf>(buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
      Ok(Self {
        {%- for field in rec.fields() %}
            {{ field.name() }}: <{{ field.type_()|ret_type_rs }} as uniffi::Liftable>::try_lift_from(buf)?,
        {%- endfor %}
      })
    }
}

impl uniffi::ViaFfiUsingByteBuffer for {{ rec.name() }} {}
