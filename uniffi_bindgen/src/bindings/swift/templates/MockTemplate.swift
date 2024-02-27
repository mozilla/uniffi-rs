{%- let obj = ci|get_object_definition(name) %}
{%- let (protocol_name, impl_class_name) = obj|object_names %}
{%- let methods = obj.methods() %}
{%- let protocol_docstring = obj.docstring() %}

{% call swift::docstring(obj, 0) %}
public class {{ impl_class_name }}Mock: {{ impl_class_name }} {

    required public init(unsafeFromRawPointer pointer: UnsafeMutableRawPointer) {
        fatalError("Not supported")
    }

    public init() {
        super.init(noPointer: NoPointer())
    }

    {%- let is_alternate_constructor = true -%}
    {%- for meth in obj.alternate_constructors() -%}
    {% include "MockFunctionTemplate.swift" %}
    {%- endfor -%}

    {%- let is_alternate_constructor = false -%}
    {%- for meth in obj.methods() -%}
    {% include "MockFunctionTemplate.swift" %}
    {%- endfor %}
}
