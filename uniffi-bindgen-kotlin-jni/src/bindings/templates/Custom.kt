{%- let type_name = custom.name_kt() %}

{%- match custom.config %}
{%- when None %}
{#- Define the type using typealiases to the builtin #}
/**
 * Typealias from the type name used in the UDL file to the builtin type.  This
 * is needed because the UDL type name is used in function/method signatures.
 * It's also what we have an external type that references a custom type.
 */
public typealias {{ type_name }} = {{ custom.builtin.type_kt }}

{%- when Some(config) %}

{# When the config specifies a different type name, create a typealias for it #}
{%- if let Some(concrete_type_name) = config.type_name %}
/**
 * Typealias from the type name used in the UDL file to the custom type.  This
 * is needed because the UDL type name is used in function/method signatures.
 * It's also what we have an external type that references a custom type.
 */
public typealias {{ type_name }} = {{ concrete_type_name }}
{%- endif %}

{%- endmatch %}
