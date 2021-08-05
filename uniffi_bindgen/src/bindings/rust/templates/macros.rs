{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_rs` in `lower_rs`)
#}

{#-
// Arglist as used in Rust declarations of methods, functions and constructors.
// Note the var_name_rs and type_rs filters.
-#}

{% macro arg_list_decl(args) %}
    {%- for arg in args -%}
        {{ arg.name()|var_name_rs }}: {{ arg.type_()|type_rs }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{% macro self_arg(method) %}
    {%- if method.takes_self_by_arc() %}
        self: Arc<Self>
    {%- else %}
        &self
    {%- endif %}
{%- endmacro %}

{% macro fallible_return_type(method, inner) %}
    {%- match method.throws() %}
    {%- when Some with (errtype) %}
        Result<{{inner}}, {{ errtype }}>
    {%- when None %}
        {{inner}}
    {%- endmatch %}
{%- endmacro %}
