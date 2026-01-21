{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list` should match up with arg lists
// passed to rust via `arg_list_lowered`
#}

{%- macro to_ffi_call(func) -%}
    {%- match func.self_type() %}
    {%- when Some(Type::Object { .. }) %}
    preventHandleFree {
        {%- call to_raw_ffi_call(func) %}
    }
    {% else %}
        {%- call to_raw_ffi_call(func) %}
    {% endmatch %}
{%- endmacro %}

{%- macro to_raw_ffi_call(func) -%}
    uniffiRun {
        {%- let args = func.full_arguments() %}
        {%- if !args.is_empty() %}
        val uniffiFfiBuffer = Memory(
            max(
                // Size needed for the arguments
                {%- for arg in args.iter() %}
                {{ arg|ffi_serializer_name(ci) }}.size(){% if !loop.last %} + {% endif %}
                {%- endfor -%}
                ,
                // Size needed for the return value
                {% if let Some(return_ty) = func.return_type() %}{{ return_ty|ffi_serializer_name(ci) }}.size() + {% endif -%}
                UniffiFfiSerializerUniffiRustCallStatus.size()
            )
        )
        {%- else %}
        val uniffiFfiBuffer = Memory(
            // Size needed for the return value
            {% if let Some(return_ty) = func.return_type() %}{{ return_ty|ffi_serializer_name(ci) }}.size() + {% endif -%}
            UniffiFfiSerializerUniffiRustCallStatus.size()
        )
        {%- endif %}
        {%- if !args.is_empty() %}
        val uniffiArgsCursor = UniffiBufferCursor(uniffiFfiBuffer)
        {%- for arg in args.iter() %}
        {{ arg|ffi_serializer_name(ci) }}.write(uniffiArgsCursor, {{ arg|lower_fn }}({{ arg|arg_name }}));
        {%- endfor %}
        {%- endif %}

        UniffiLib.{{ func.ffi_func().pointer_ffi_name() }}(uniffiFfiBuffer)
        val uniffiReturnCursor = UniffiBufferCursor(uniffiFfiBuffer)
        val uniffiCallStatus = UniffiFfiSerializerUniffiRustCallStatus.read(uniffiReturnCursor)

        {%- if let Some(throws_ty) = func.throws_type() %}
        {%- if ci.is_external(throws_ty) %}
        uniffiCheckCallStatus({{ throws_ty|type_name(ci) }}ExternalErrorHandler, uniffiCallStatus)
        {%- else %}
        uniffiCheckCallStatus({{ throws_ty|type_name(ci) }}, uniffiCallStatus)
        {%- endif %}
        {%- else %}
        uniffiCheckCallStatus(UniffiNullRustCallStatusErrorHandler, uniffiCallStatus)
        {%- endif %}

        {%- if let Some(return_ty) = func.return_type() %}
        {{ return_ty|ffi_serializer_name(ci) }}.read(uniffiReturnCursor)
        {%- endif %}
    }
{%- endmacro -%}

{%- macro func_decl(func_decl, callable, indent) %}
    {%- call docstring(callable, indent) %}

    {%- match callable.throws_type() -%}
    {%-     when Some(throwable) %}
    @Throws({{ throwable|type_name(ci) }}::class)
    {%-     else -%}
    {%- endmatch -%}
    {%- if callable.is_async() %}
    @Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE")
    {{ func_decl }} suspend fun {{ callable.name()|fn_name }}(
        {%- call arg_list(callable, callable.self_type().is_none()) -%}
    ){% match callable.return_type() %}{% when Some(return_type) %} : {{ return_type|type_name(ci) }}{% when None %}{%- endmatch %} {
        {% call call_async(callable) %}
    }
    {%- else -%}
    {{ func_decl }} fun {{ callable.name()|fn_name }}(
        {%- call arg_list(callable, callable.self_type().is_none()) -%}
    ){%- match callable.return_type() -%}
    {%-         when Some(return_type) -%}
        : {{ return_type|type_name(ci) }} {
            return {{ return_type|lift_fn }}({% call to_ffi_call(callable) %})
    }
    {%-         when None %}
        = {% call to_ffi_call(callable) %}
    {%-     endmatch %}
    {% endif %}
{% endmacro %}

{%- macro call_async(callable) -%}
    {%- match callable.self_type() %}
    {%- when Some(Type::Object { .. }) %}
    val uniffiRustFuture = preventHandleFree {
        {%- call to_raw_ffi_call_async(callable) %}
    }
    {% else %}
    val uniffiRustFuture = {%- call to_raw_ffi_call_async(callable) %}
    {% endmatch %}

    val uniffiReturnBufSize = UniffiFfiSerializerUniffiRustCallStatus.size()
        {%- if let Some(return_ty) = callable.return_type() %} + {{ return_ty|ffi_serializer_name(ci) }}.size(){% endif %}

    return uniffiDriveRustFutureToCompletion(uniffiRustFuture, uniffiReturnBufSize) { uniffiFfiBuffer ->
        val uniffiCallStatus = UniffiFfiSerializerUniffiRustCallStatus.read(uniffiFfiBuffer)

        {%- if let Some(throws_ty) = callable.throws_type() %}
        {%- if ci.is_external(throws_ty) %}
        uniffiCheckCallStatus({{ throws_ty|type_name(ci) }}ExternalErrorHandler, uniffiCallStatus)
        {%- else %}
        uniffiCheckCallStatus({{ throws_ty|type_name(ci) }}, uniffiCallStatus)
        {%- endif %}
        {%- else %}
        uniffiCheckCallStatus(UniffiNullRustCallStatusErrorHandler, uniffiCallStatus)
        {%- endif %}

        {%- if let Some(return_ty) = callable.return_type() %}
        {{ return_ty|lift_fn }}(
            {{ return_ty|ffi_serializer_name(ci) }}.read(uniffiFfiBuffer)
        )
        {%- endif %}
    }
{%- endmacro %}

{%- macro to_raw_ffi_call_async(func) -%}
    uniffiRun {
        {%- let args = func.full_arguments() %}
        {%- if !args.is_empty() %}
        val uniffiFfiBuffer = Memory(
            max(
                // Size needed for the arguments
                {%- for arg in args.iter() %}
                {{ arg|ffi_serializer_name(ci) }}.size(){% if !loop.last %} + {% endif %}
                {%- endfor -%}
                ,
                // Size needed for the return value
                {% if let Some(return_ty) = func.return_type() %}{{ return_ty|ffi_serializer_name(ci) }}.size() + {% endif -%}
                UniffiFfiSerializerUniffiRustCallStatus.size()
            )
        )
        {%- else %}
        val uniffiFfiBuffer = Memory(
            // Size needed for the return value
            {% if let Some(return_ty) = func.return_type() %}{{ return_ty|ffi_serializer_name(ci) }}.size() + {% endif -%}
            UniffiFfiSerializerUniffiRustCallStatus.size()
        )
        {%- endif %}

        {%- if !args.is_empty() %}
        val uniffiArgsCursor = UniffiBufferCursor(uniffiFfiBuffer)
        {%- for arg in args.iter() %}
        {{ arg|ffi_serializer_name(ci) }}.write(uniffiArgsCursor, {{ arg|lower_fn }}({{ arg|arg_name }}));
        {%- endfor %}
        {%- endif %}

        UniffiLib.{{ func.ffi_func().pointer_ffi_name() }}(uniffiFfiBuffer)

        // Async functions always return a future handle, regardless of the return value of the
        // actual function
        val uniffiReturnCursor = UniffiBufferCursor(uniffiFfiBuffer)
        UniffiFfiSerializerHandle.read(uniffiReturnCursor)
    }
{%- endmacro -%}


{%- macro arg_list_lowered(func) %}
    {%- for arg in func.arguments() %}
        {{- arg|lower_fn }}({{ arg.name()|var_name }}),
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in kotlin declarations of methods, functions and constructors.
// If is_decl, then default values be specified.
// Note the var_name and type_name filters.
-#}

{% macro arg_list(func, is_decl) %}
{%- for arg in func.arguments() -%}
        {{ arg.name()|var_name }}: {{ arg|type_name(ci) }}
{%-     if is_decl %}
{%-         match arg.default_value() %}
{%-             when Some(default) %} = {{ default|render_default(arg, ci) }}
{%-             else %}
{%-         endmatch %}
{%-     endif %}
{%-     if !loop.last %}, {% endif -%}
{%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the UniffiLib function declarations.
// Note unfiltered name but ffi_type_name filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|var_name }}: {{ arg.type_().borrow()|ffi_type_name(ci) -}},
    {%- endfor %}
    {%- if func.has_rust_call_status_arg() %}uniffi_out_err: UniffiRustCallStatus, {% endif %}
{%- endmacro -%}

{% macro field_name(field, field_num) %}
{%- if field.name().is_empty() -%}
v{{- field_num -}}
{%- else -%}
{{ field.name()|var_name }}
{%- endif -%}
{%- endmacro %}

{% macro field_name_unquoted(field, field_num) %}
{%- if field.name().is_empty() -%}
v{{- field_num -}}
{%- else -%}
{{ field.name()|var_name|unquote }}
{%- endif -%}
{%- endmacro %}

 // Macro for destroying fields
{%- macro destroy_fields(member) %}
    Disposable.destroy(
    {%- for field in member.fields() %}
        this.{%- call field_name(field, loop.index) -%}{% if loop.last %}{% else %},{% endif -%}
    {%- endfor %}
    )
{%- endmacro -%}

{%- macro docstring_value(maybe_docstring, indent_spaces) %}
{%- match maybe_docstring %}
{%- when Some(docstring) %}
{{ docstring|docstring(indent_spaces) }}
{%- else %}
{%- endmatch %}
{%- endmacro %}

{%- macro docstring(defn, indent_spaces) %}
{%- call docstring_value(defn.docstring(), indent_spaces) %}
{%- endmacro %}

// macro for uniffi_trait implementations.
{% macro uniffi_trait_impls(uniffi_trait_methods) %}
{# We have 2 display traits, kotlin has 1. Prefer `Display` but use `Debug` otherwise #}
{%- if let Some(fmt) = uniffi_trait_methods.display_fmt.or(uniffi_trait_methods.debug_fmt.clone()) %}
    // The local Rust `Display`/`Debug` implementation.
    override fun toString(): String {
        return {{ fmt.return_type().unwrap()|lift_fn }}({% call to_ffi_call(fmt) %})
    }
{%- endif %}
{%- if let Some(eq) = uniffi_trait_methods.eq_eq %}
    // The local Rust `Eq` implementation - only `eq` is used.
    override fun equals(other: Any?): Boolean {
        if (other !is {{ eq.object_name()|class_name(ci) }}) return false
        return {{ eq.return_type().unwrap()|lift_fn }}({% call to_ffi_call(eq) %})
    }
{%- endif %}
{%- if let Some(hash) = uniffi_trait_methods.hash_hash %}
    // The local Rust `Hash` implementation
    override fun hashCode(): Int {
        return {{ hash.return_type().unwrap()|lift_fn }}({%- call to_ffi_call(hash) %}).toInt()
    }
{%- endif %}
{%- if let Some(cmp) = uniffi_trait_methods.ord_cmp %}
    // The local Rust `Ord` implementation
    override fun compareTo(other: {{ cmp.object_name()|class_name(ci) }}): Int {
        return {{ cmp.return_type().unwrap()|lift_fn }}({%- call to_ffi_call(cmp) %}).toInt()
    }
{%- endif %}
{%- endmacro %}
