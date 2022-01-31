{%- import "macros.kt" as kt -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_name %}
{%- let canonical_type_name = outer_type|canonical_name %}

internal object {{ outer_type|ffi_converter_name }} {
    internal fun lower(m: Map<String, {{ inner_type_name }}>): RustBuffer.ByValue {
        return lowerIntoRustBuffer(m) { m, buf ->
            write(m, buf)
        }
    }

    internal fun write(v: Map<String, {{ inner_type_name }}>, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        // The parens on `(k, v)` here ensure we're calling the right method,
        // which is important for compatibility with older android devices.
        // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
        v.forEach { (k, v) ->
            {{ TypeIdentifier::String|write_fn }}(k, buf)
            {{ inner_type|write_fn }}(v, buf)
        }
    }

    internal fun lift(rbuf: RustBuffer.ByValue): Map<String, {{ inner_type_name }}> {
        return liftFromRustBuffer(rbuf) { buf ->
            read(buf)
        }
    }

    internal fun read(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
        // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
        val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
        val len = buf.getInt()
        repeat(len) {
            val k = {{ TypeIdentifier::String|read_fn }}(buf)
            val v = {{ inner_type|read_fn }}(buf)
            items[k] = v
        }
        return items
    }
}
