{% import "macros.kt" as kt %}
{%- let obj = self.inner() %}
{%- let generic_return_type = "ReturnType" %}
{%- let generic_object_type = "ObjectType" %}
public interface {{ obj.name()|class_name_kt }}<{{generic_object_type}}> {
    {% for meth in obj.methods() -%}
    fun <{{generic_return_type}}> {{ meth.name()|fn_name_kt }}(obj: {{generic_object_type}}, thunk: () -> {{generic_return_type}})
        {%- match meth.return_type() -%}
            {%- when Some with (return_type) %}: {{ return_type|type_t_kt(generic_return_type) -}}
            {%- when None -%}
        {%- endmatch %}
    {% endfor %}
}
