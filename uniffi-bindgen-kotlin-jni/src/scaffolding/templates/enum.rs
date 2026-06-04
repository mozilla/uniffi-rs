{%- let type_name = en.self_type.type_rs %}

{%- if !en.is_flat_error() %}
unsafe fn {{ en.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    uniffi_value: {{ type_name }},
) -> uniffi::Result<{{ en.self_type.lowered_type_rs() }}> {
    unsafe {
        match uniffi_value {
            {%- for v in en.variants %}
            {%- match v.fields_kind %}
            {%- when FieldsKind::Unit %}
            {{ type_name }}::{{ v.name_rs() }} => {
            {%- when FieldsKind::Named %}
            {{ type_name }}::{{ v.name_rs() }} {
                {%- for f in v.fields %}
                {{ f.name_rs() }}: uniffi_field{{ f.index}},
                {%- endfor %}
            } => {
            {%- when FieldsKind::Unnamed %}
            {{ type_name }}::{{ v.name_rs() }} (
                {%- for f in v.fields %}
                uniffi_field{{ f.index }},
                {%- endfor %}
            ) => {
            {%- endmatch %}
                // The discriminant is always the first FFI field
                let uniffi_ffi_field_0 = {{ loop.index0 }};

                {%- for f in v.fields %}
                {%- if f.lowers_to_primitive() %}
                let uniffi_ffi_field_{{ f.ffi_fields[0].index }} = {{ f.ty.lower_fn_rs() }}(uniffi_env, uniffi_field{{ f.index }})?;
                {%- else %}
                let (
                    {%- for ffi_field in f.ffi_fields %}
                    uniffi_ffi_field_{{ ffi_field.index }},
                    {%- endfor %}
                ) = {{ f.ty.lower_fn_rs() }}(uniffi_env, uniffi_field{{ f.index }})?;
                {%- endif %}
                {%- endfor %}

                {%- if en.self_type.lowers_to_primitive() %}
                uniffi::Result::Ok(uniffi_ffi_field_0)
                {%- else %}
                uniffi::Result::Ok((
                    {%- for ffi_field in en.ffi_fields %}
                    {%- if v.used_ffi_fields.contains(*ffi_field) %}
                    uniffi_ffi_field_{{ ffi_field.index }},
                    {%- else %}
                    <{{ ffi_field.ty.type_rs() }} as ::std::default::Default>::default(),
                    {%- endif %}
                    {%- endfor %}
                ))
                {%- endif %}
            }
            {%- endfor %}
        }
    }
}

unsafe fn {{ en.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    {%- for ffi_field in en.ffi_fields %}
    v{{ ffi_field.index }}: {{ ffi_field.ty.type_rs() }},
    {%- endfor %}
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        match v0 {
            {%- for v in en.variants %}
            {{ loop.index0 }} => {
                {%- for field in v.fields %}
                let uniffi_field{{ field.index }} = {{ field.ty.lift_fn_rs() }}(
                    uniffi_env,
                    {%- for ffi_field in field.ffi_fields %}
                    v{{ ffi_field.index }},
                    {%- endfor %}
                )?;
                {%- endfor %}

                {%- match v.fields_kind %}
                {%- when FieldsKind::Unit %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }})
                {%- when FieldsKind::Named %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }} {
                    {%- for field in v.fields %}
                    {{ field.name_rs() }}: uniffi_field{{ field.index }},
                    {%- endfor %}
                })
                {%- when FieldsKind::Unnamed %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }}(
                    {%- for field in v.fields %}
                    uniffi_field{{ field.index }},
                    {%- endfor %}
                ))
                {%- endmatch %}
            }
            {%- endfor %}
            d => uniffi::deps::anyhow::bail!("{{ en.self_type.lift_fn_rs() }}: invalid discriminant: {d}"),
        }
    }
}

unsafe fn {{ en.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    unsafe {
        match value {
            {%- for v in en.variants %}
            {%- match v.fields_kind %}
            {%- when FieldsKind::Unit %}
            {{ type_name }}::{{ v.name_rs() }} => {
            {%- when FieldsKind::Named %}
            {{ type_name }}::{{ v.name_rs() }} {
                {%- for f in v.fields %}
                {{ f.name_rs() }}: uniffi_field{{ f.index}},
                {%- endfor %}
            } => {
            {%- when FieldsKind::Unnamed %}
            {{ type_name }}::{{ v.name_rs() }} (
                {%- for f in v.fields %}
                uniffi_field{{ f.index }},
                {%- endfor %}
            ) => {
            {%- endmatch %}
                uniffi::ffibuffer::write_i32(ptr, {{ loop.index0 }})?;
                {%- for f in v.fields %}
                {{ f.ty.write_fn_rs() }}(ptr.add({{ f.offset }}), uniffi_field{{ f.index }})?;
                {%- endfor %}
                uniffi::Result::Ok(())
            }
            {%- endfor %}
        }
    }
}

unsafe fn {{ en.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        let discriminant = uniffi::ffibuffer::read_i32(ptr)?;
        match discriminant {
            {%- for v in en.variants %}
            {{ loop.index0 }} => {
                {%- for field in v.fields %}
                let uniffi_field{{ field.index }} = {{ field.ty.read_fn_rs() }}(
                    ptr.add({{ field.offset }}),
                )?;
                {%- endfor %}

                {%- match v.fields_kind %}
                {%- when FieldsKind::Unit %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }})
                {%- when FieldsKind::Named %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }} {
                    {%- for field in v.fields %}
                    {{ field.name_rs() }}: uniffi_field{{ field.index }},
                    {%- endfor %}
                })
                {%- when FieldsKind::Unnamed %}
                uniffi::Result::Ok({{ type_name }}::{{ v.name_rs() }}(
                    {%- for field in v.fields %}
                    uniffi_field{{ field.index }},
                    {%- endfor %}
                ))
                {%- endmatch %}
            }
            {%- endfor %}
            d => uniffi::deps::anyhow::bail!("{{ en.self_type.read_fn_rs() }}: invalid discriminent: {d}"),
        }
    }
}

{%- else %}
unsafe fn {{ en.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<{{ en.self_type.lowered_type_rs() }}> {
    unsafe {
        let msg = <{{ type_name }} as ::std::string::ToString>::to_string(&value);
        let jstring = uniffi_jni::lower_string(uniffi_env, msg)?;
        match value {
            {%- for v in en.variants %}
            {{ type_name }}::{{ v.name_rs() }} { .. } => {
                uniffi::Result::Ok(({{ loop.index0 }}, jstring))
            }
            {%- endfor %}
        }
    }
}

unsafe fn {{ en.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    unsafe {
        let msg = <{{ type_name }} as ::std::string::ToString>::to_string(&value);
        match value {
            {%- for v in en.variants %}
            {{ type_name }}::{{ v.name_rs() }} { .. } => {
                uniffi::ffibuffer::write_i32(ptr, {{ loop.index0 }})?;
                uniffi::ffibuffer::write_string(ptr.add(8), msg)?;
                uniffi::Result::Ok(())
            }
            {%- endfor %}
        }
    }
}

{#
 # No lift/read functions for flat errors.
 # We only support passing them from Rust -> Kotlin.
 #}

{%- endif %}
