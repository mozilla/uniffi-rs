{%- import "macros.kt" as kt -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_name %}
{%- let canonical_type_name = outer_type|canonical_name %}

internal object {{ outer_type|ffi_converter_name }} {
    fun lift(rbuf: RustBuffer.ByValue): {{ inner_type_name }}? {
        return liftFromRustBuffer(rbuf) { buf ->
            read(buf)
        }
    }

    fun read(buf: ByteBuffer): {{ inner_type_name }}? {
        if (buf.get().toInt() == 0) {
            return null
        }
        return {{ inner_type|read_fn }}(buf)
    }

    fun lower(v: {{ inner_type_name }}?): RustBuffer.ByValue {
        return lowerIntoRustBuffer(v) { v, buf ->
            write(v, buf)
        }
    }

    fun write(v: {{ inner_type_name }}?, buf: RustBufferBuilder) {
        if (v == null) {
            buf.putByte(0)
        } else {
            buf.putByte(1)
            {{ inner_type|write_fn }}(v, buf)
        }
    }
}
