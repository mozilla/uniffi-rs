{%- let type_name = en.self_type.type_kt %}

{%- match en.kotlin_kind %}
{%- when KotlinEnumKind::EnumClass { .. } %}
@JvmName("{{ en.self_type.lower_fn_kt() }}")
fun {{ en.self_type.lower_fn_kt() }}(value: {{ type_name }}): kotlin.Int {
    return value.ordinal
}

@JvmName("{{ en.self_type.lift_fn_kt() }}")
fun {{ en.self_type.lift_fn_kt() }}(value: kotlin.Int): {{ type_name }} {
    {%- if en.use_entries %}
    return {{ type_name }}.entries[value]
    {%- else %}
    return {{ type_name }}.values()[value]
    {%- endif %}
}

fun {{ en.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    writeInt(buf, offset, value.ordinal)
}

fun {{ en.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    return {{ en.self_type.lift_fn_kt() }}(readInt(buf, offset))
}

{%- when KotlinEnumKind::SealedClass %}
{%- if !en.self_type.lowers_to_primitive() %}
// Deconstructed version of {{ en.name }}
class {{ en.self_type.lowered_type_kt() }}(
    {%- for ffi_field in en.ffi_fields %}
    val v{{ ffi_field.index }}: {{ ffi_field.ty.type_kt() }},
    {%- endfor %}
)
{%- endif %}

@JvmName("{{ en.self_type.lower_fn_kt() }}")
fun {{ en.self_type.lower_fn_kt() }}(value: {{ type_name }}): {{ en.self_type.lowered_type_kt() }} {
    when (value) {
        {%- for v in en.variants %}
        is {{ type_name }}.{{ v.name_kt }} -> {
            // The discriminant is always the first FFI field
            val uniffiFieldLowered0 = {{ loop.index0 }}

            {%- for field in v.fields %}
            {%- if field.lowers_to_primitive() %}
            val uniffiFieldLowered{{ field.ffi_fields[0].index }} = {{ field.ty.lower_fn_kt() }}(value.{{ field.name_kt() }})
            {%- else %}
            val uniffiFieldDeconstructed{{ field.index }} = {{ field.ty.lower_fn_kt() }}(value.{{ field.name_kt() }})
            {%- for ffi_field in field.ffi_fields %}
            val uniffiFieldLowered{{ ffi_field.index }} = uniffiFieldDeconstructed{{ field.index }}.v{{ loop.index0 }}
            {%- endfor %}
            {%- endif %}
            {%- endfor %}

            {%- if en.self_type.lowers_to_primitive() %}
            return uniffiFieldLowered0
            {%- else %}
            return {{ en.self_type.lowered_type_kt() }}(
                {%- for ffi_field in en.ffi_fields %}
                {%- if v.used_ffi_fields.contains(*ffi_field) %}
                uniffiFieldLowered{{ ffi_field.index }},
                {%- else %}
                {{ ffi_field.ty.default_kt() }},
                {%- endif %}
                {%- endfor %}
            )
            {%- endif %}
        }
        {%- endfor %}
    }
}

@JvmName("{{ en.self_type.lift_fn_kt() }}")
fun {{ en.self_type.lift_fn_kt() }}(
    {%- for ffi_field in en.ffi_fields %}
    v{{ ffi_field.index }}: {{ ffi_field.ty.type_kt() }},
    {%- endfor %}
): {{ type_name }} {
    when (v0) {
        {%- for v in en.variants %}
        {{ loop.index0 }} -> {
            {%- if v.fields.is_empty() && !en.self_type.is_used_as_error %}
            return {{ type_name }}.{{ v.name_kt }}
            {%- else %}
            return {{ type_name }}.{{ v.name_kt }}(
                {%- for field in v.fields %}
                {{ field.ty.lift_fn_kt() }}(
                    {%- for ffi_field in field.ffi_fields %}
                    v{{ ffi_field.index }},
                    {%- endfor %}
                ),
                {%- endfor %}
            )
            {%- endif %}
        }
        {%- endfor %}
        else -> {
            throw uniffi.InternalException("{{ en.self_type.lift_fn_kt() }}: Invalid enum discriminant: ${v0}")
        }
    }
}

fun {{ en.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    when (value) {
        {%- for v in en.variants %}
        is {{ type_name }}.{{ v.name_kt }} -> {
            writeInt(buf, offset, {{ loop.index0 }});
            {%- for f in v.fields %}
            {{ f.ty.write_fn_kt() }}(buf, offset + {{ f.offset }}, value.{{ f.name_kt() }})
            {%- endfor %}
        }
        {%- endfor %}
    }
}

fun {{ en.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val discriminant = readInt(buf, offset);
    return when (discriminant) {
        {%- for v in en.variants %}
        {{ loop.index0 }} -> {
            {%- if v.fields.is_empty() && !en.self_type.is_used_as_error %}
            {{ type_name }}.{{ v.name_kt }}
            {%- else %}
            {{ type_name }}.{{ v.name_kt }}(
                {%- for f in v.fields %}
                {{ f.ty.read_fn_kt() }}(buf, offset + {{ f.offset }}),
                {%- endfor %}
            )
            {%- endif %}
        }
        {%- endfor %}
        else -> {
            throw uniffi.InternalException("{{ en.self_type.lift_fn_kt() }}: Invalid enum discriminant: ${discriminant}")
        }
    }
}

{%- when KotlinEnumKind::FlatError %}
@JvmName("{{ en.self_type.lift_fn_kt() }}")
fun {{ en.self_type.lift_fn_kt() }}(v0: kotlin.Int, v1: kotlin.String): {{ type_name }} {
    return when(v0) {
        {%- for v in en.variants %}
        {{ loop.index0 }} -> {{ type_name }}.{{ v.name_kt }}(v1)
        {%- endfor %}
        else -> throw uniffi.InternalException("{{ en.self_type.lift_fn_kt() }}: Invalid enum value: ${v0}")
    }
}


fun {{ en.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val uniffiDiscriminent = uniffi.readInt(buf, offset)
    return when(uniffiDiscriminent) {
        {%- for v in en.variants %}
        {{ loop.index0 }} -> {{ type_name }}.{{ v.name_kt }}(readString(buf, offset + 8))
        {%- endfor %}
        else -> throw uniffi.InternalException("{{ en.self_type.read_fn_kt() }}: Invalid enum value: ${uniffiDiscriminent}")
    }
}

{# No lower/write functions, passing flat errors back to Rust is not supported #}
{%- endmatch %}
