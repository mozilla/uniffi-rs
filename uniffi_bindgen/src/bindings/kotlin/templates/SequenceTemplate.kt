{%- import "macros.kt" as kt -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_kt %}
{%- let canonical_type_name = outer_type|canonical_name %}

// Helper functions for passing values of type {{ outer_type|type_kt }}
internal fun lower{{ canonical_type_name }}(v: List<{{ inner_type_name }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: List<{{ inner_type_name }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    v.forEach {
        {{ "it"|write_kt("buf", inner_type) }}
    }
}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): List<{{ inner_type_name }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): List<{{ inner_type_name }}> {
    val len = buf.getInt()
    return List<{{ inner_type_name }}>(len) {
        {{ "buf"|read_kt(inner_type) }}
    }
}