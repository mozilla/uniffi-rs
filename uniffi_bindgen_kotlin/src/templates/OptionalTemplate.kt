{%- import "macros.kt" as kt -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_name %}
{%- let canonical_type_name = outer_type|canonical_name %}

public object {{ outer_type|ffi_converter_name }}: FfiConverterRustBuffer<{{ inner_type_name }}?> {
    override fun read(buf: ByteBuffer): {{ inner_type_name }}? {
        if (buf.get().toInt() == 0) {
            return null
        }
        return {{ inner_type|read_fn }}(buf)
    }

    override fun allocationSize(value: {{ inner_type_name }}?): Int {
        if (value == null) {
            return 1
        } else {
            return 1 + {{ inner_type|allocation_size_fn }}(value)
        }
    }

    override fun write(value: {{ inner_type_name }}?, buf: ByteBuffer) {
        if (value == null) {
            buf.put(0)
        } else {
            buf.put(1)
            {{ inner_type|write_fn }}(value, buf)
        }
    }
}
