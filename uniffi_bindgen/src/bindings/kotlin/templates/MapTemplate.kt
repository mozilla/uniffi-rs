{%- import "macros.kt" as kt -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_kt %}
{%- let canonical_type_name = outer_type|canonical_name %}

// Helper functions for passing values of type {{ outer_type|type_kt }}
internal fun lower{{ canonical_type_name }}(m: Map<String, {{ inner_type_name }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(m) { m, buf ->
        write{{ canonical_type_name }}(m, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: Map<String, {{ inner_type_name }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    // The parens on `(k, v)` here ensure we're calling the right method,
    // which is important for compatibility with older android devices.
    // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
    v.forEach { (k, v) ->
        {{ "k"|write_kt("buf", TypeIdentifier::String) }}
        {{ "v"|write_kt("buf", inner_type) }}
    }
}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): Map<String, {{ inner_type_name }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
    // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
    val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
    val len = buf.getInt()
    repeat(len) {
        val k = {{ "buf"|read_kt(TypeIdentifier::String) }}
        val v = {{ "buf"|read_kt(inner_type) }}
        items[k] = v
    }
    return items
}