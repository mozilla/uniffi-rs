{%- let type_name = cls.self_type.type_kt %}
{%- let impl_class_name = "{}.{}"|format(cls.package_name, cls.name_kt()) %}

{%- match cls.imp %}
{%- when ObjectImpl::Struct %}
{# Rust struct #}

fun {{ cls.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ impl_class_name }}(uniffi.WithHandle, uniffi.readLong(cursor))
}

fun {{ cls.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    value.uniffiAddRef()
    uniffi.writeLong(cursor, value.uniffiHandle)
}
{%- when ObjectImpl::Trait %}
{#
 # Rust trait.
 # These store 2 handles, which corresponds to a wide pointer on the Rust side.
 #}

fun {{ cls.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ impl_class_name }}(uniffi.WithHandle, uniffi.readLong(cursor), uniffi.readLong(cursor))
}

fun {{ cls.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ impl_class_name }}) {
    value.uniffiAddRef()
    uniffi.writeLong(cursor, value.uniffiHandle)
    uniffi.writeLong(cursor, value.uniffiHandle2)
}

{%- when ObjectImpl::CallbackTrait %}
{#
 # Rust trait that can also be implemented in Kotlin
 # These also store 2 handles.
 # For Kotlin callback interfaces, the first handle is `0`
 #}

fun {{ cls.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val handle = uniffi.readLong(cursor)
    val handle2 = uniffi.readLong(cursor)
    if (handle == 0L) {
        return {{ cls.handle_map_kt() }}.remove(handle2)
    } else {
        return {{ impl_class_name }}(uniffi.WithHandle, handle, handle2)
    }
}

fun {{ cls.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    if (value is {{ impl_class_name }}) {
        value.uniffiAddRef()
        uniffi.writeLong(cursor, value.uniffiHandle)
        uniffi.writeLong(cursor, value.uniffiHandle2)
     } else {
        val handle = {{ cls.handle_map_kt() }}.insert(value)
        uniffi.writeLong(cursor, 0)
        uniffi.writeLong(cursor, handle)
     }
}

{%- endmatch %}
