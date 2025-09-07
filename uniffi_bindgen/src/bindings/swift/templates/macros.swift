{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `arg_list_lowered`
#}

{%- macro to_ffi_call(func) -%}
    {%- call is_try(func) -%}
    {%- if let(Some(e)) = func.throws_type() -%}
        rustCallWithError({{ e|ffi_error_converter_name }}_lift) {
    {%- else -%}
        rustCall() {
    {%- endif %}
    {{ func.ffi_func().name() }}(
        {%- match func.self_type() %}
        {%-     when Some(Type::Object { .. }) %}
            self.uniffiCloneHandle(),
        {%-     when Some(t) %}
            {{ t|lower_fn }}(self),
        {%-     when None %}
        {%- endmatch %}
        {%- call arg_list_lowered(func) -%} $0
    )
}
{%- endmacro -%}

// eg, `public func foo_bar() { body }`
{%- macro func_decl(func_decl, callable, indent) %}
{%- call docstring(callable, indent) %}
{{ func_decl }} {{ callable.name()|fn_name }}(
    {%- call arg_list_decl(callable) -%})
    {%- call is_async(callable) %}
    {%- call throws(callable) %}
    {%- if let Some(return_type) = callable.return_type() %} -> {{ return_type|type_name }} {%- endif %}  {
    {%- call call_body(callable) %}
}
{%- endmacro %}

// primary ctor - no name, no return-type.
{%- macro ctor_decl(callable, indent) %}
{%- call docstring(callable, indent) %}
public convenience init(
    {%- call arg_list_decl(callable) -%}) {%- call is_async(callable) %} {%- call throws(callable) %} {
    {%- if callable.is_async() %}
    let handle =
        {%- call call_async(callable) %}
        {# The async mechanism returns an already constructed self.
           We work around that by cloning the handle from that object, then
           assume the old object dies as there are no other references possible.
        #}
        .uniffiCloneHandle()
    {%- else %}
    let handle =
        {% call to_ffi_call(callable) %}
    {%- endif %}
    self.init(unsafeFromHandle: handle)
}
{%- endmacro %}

{%- macro call_body(callable) %}
{%- if callable.is_async() %}
    return {%- call call_async(callable) %}
{%- else %}
{%-     match callable.return_type() -%}
{%-         when Some(return_type) %}
    return {% call is_try(callable) %} {{ return_type|lift_fn }}({% call to_ffi_call(callable) %})
{%-         when None %}
{%-             call to_ffi_call(callable) %}
{%-     endmatch %}
{%- endif %}

{%- endmacro %}

{%- macro call_async(callable) %}
        {% call is_try(callable) %} await uniffiRustCallAsync(
            rustFutureFunc: {
                {{ callable.ffi_func().name() }}(
                    {%- if callable.self_type().is_some() %}
                    self.uniffiCloneHandle(){% if !callable.arguments().is_empty() %},{% endif %}
                    {% endif %}
                    {%- for arg in callable.arguments() -%}
                    {{ arg|lower_fn }}({{ arg.name()|var_name }}){% if !loop.last %},{% endif %}
                    {%- endfor %}
                )
            },
            pollFunc: {{ callable.ffi_rust_future_poll(ci) }},
            completeFunc: {{ callable.ffi_rust_future_complete(ci) }},
            freeFunc: {{ callable.ffi_rust_future_free(ci) }},
            {%- match callable.return_type() %}
            {%- when Some(return_type) %}
            liftFunc: {{ return_type|lift_fn }},
            {%- when None %}
            liftFunc: { $0 },
            {%- endmatch %}
            {%- match callable.throws_type() %}
            {%- when Some with (e) %}
            errorHandler: {{ e|ffi_error_converter_name }}_lift
            {%- else %}
            errorHandler: nil
            {% endmatch %}
        )
{%- endmacro %}

{%- macro arg_list_lowered(func) %}
    {%- for arg in func.arguments() %}
        {{ arg|lower_fn }}({{ arg.name()|var_name }}),
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Swift declarations of methods, functions and constructors.
// Note the var_name and type_name filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {% if config.omit_argument_labels() %}_ {% endif %}{{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- match arg.default_value() %}
        {%- when Some with(default) %} = {{ default|default_swift(arg) }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Field lists as used in Swift declarations of Records and Enums.
// Note the var_name and type_name filters.
-#}
{% macro field_list_decl(item, has_nameless_fields) %}
    {%- for field in item.fields() -%}
        {%- call docstring(field, 8) %}
        {%- if has_nameless_fields %}
        {{- field|type_name -}}
        {%- if !loop.last -%}, {%- endif -%}
        {%- else -%}
        {{ field.name()|var_name }}: {{ field|type_name -}}
        {%- match field.default_value() %}
            {%- when Some with(default) %} = {{ default|default_swift(field) }}
            {%- else %}
        {%- endmatch -%}
        {% if !loop.last %}, {% endif %}
        {%- endif -%}
    {%- endfor %}
{%- endmacro %}

{% macro field_name(field, field_num) %}
{%- if field.name().is_empty() -%}
v{{- field_num -}}
{%- else -%}
{{ field.name()|var_name }}
{%- endif -%}
{%- endmacro %}

{% macro arg_list_protocol(func) %}
    {%- for arg in func.arguments() -%}
        {% if config.omit_argument_labels() %}_ {% endif %}{{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{%- macro is_async(func) %}
{%- if func.is_async() %}async {% endif %}
{%- endmacro -%}

{%- macro throws(func) %}
{%- if func.throws() %}throws {% endif %}
{%- endmacro -%}

{%- macro is_try(func) %}
{%- if func.throws() %}try {% else %}try! {% endif %}
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
{%- if let Some(fmt) = uniffi_trait_methods.debug_fmt %}
// The local Rust `Debug` implementation.
extension {{ fmt.object_name() }}: CustomDebugStringConvertible {
    public var debugDescription: String {
        return {% call is_try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call to_ffi_call(fmt) %}
        )
    }
}
{%- endif %}
{%- if let Some(fmt) = uniffi_trait_methods.display_fmt %}
// The local Rust `Display` implementation.
extension {{ fmt.object_name() }}: CustomStringConvertible {
    public var description: String {
        return {% call is_try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call to_ffi_call(fmt) %}
        )
    }
}
{%- endif %}
{%- if let Some(eq) = uniffi_trait_methods.eq_eq %}
// The local Rust `Eq` implementation - only `eq` is used.
extension {{ eq.object_name() }}: Equatable {
    public static func == (self: {{ eq.object_name() }}, other: {{ eq.object_name() }}) -> Bool {
        return {% call is_try(eq) %} {{ eq.return_type().unwrap()|lift_fn }}(
            {% call to_ffi_call(eq) %}
        )
    }
}
{%- endif %}
{%- if let Some(hash) = uniffi_trait_methods.hash_hash %}
// The local Rust `Hash` implementation
extension {{ hash.object_name() }}: Hashable {
    public func hash(into hasher: inout Hasher) {
        let val = {% call is_try(hash) %} {{ hash.return_type().unwrap()|lift_fn }}(
            {% call to_ffi_call(hash) %}
        )
        hasher.combine(val)
    }
}
{%- endif %}
{%- if let Some(cmp) = uniffi_trait_methods.ord_cmp %}
// The local Rust `Ord` implementation
extension {{ cmp.object_name() }}: Comparable {
    public static func < (self: {{ cmp.object_name() }}, other: {{ cmp.object_name() }}) -> Bool {
        return {% call is_try(cmp) %} {{ cmp.return_type().unwrap()|lift_fn }}(
            {% call to_ffi_call(cmp) %}
        ) < 0
    }
}
{%- endif %}
{%- endmacro %}
