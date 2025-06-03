{%- let type_name = e.self_type.type_name %}
# {{ type_name }}
# We want to define each variant as a nested class that's also a subclass,
# which is tricky in Python.  To accomplish this we're going to create each
# class separately, then manually add the child classes to the base class's
# __dict__.  All of this happens in dummy class to avoid polluting the module
# namespace.
class {{ type_name }}(Exception):
    {{ e.docstring|docstring(4) -}}
    pass

_UniffiTemp{{ type_name }} = {{ type_name }}

class {{ type_name }}:  # type: ignore
    {{ e.docstring|docstring(4) -}}
    {%- for variant in e.variants -%}
    {%- let variant_type_name = variant.name -%}
    {%- if e.is_flat %}
    class {{ variant_type_name }}(_UniffiTemp{{ type_name }}):
        {{ variant.docstring|docstring(8) -}}

        def __repr__(self):
            return "{{ type_name }}.{{ variant_type_name }}({})".format(repr(str(self)))
    {%- else %}
    class {{ variant_type_name }}(_UniffiTemp{{ type_name }}):
        {{ variant.docstring|docstring(8) -}}

    {%-     if variant.has_unnamed_fields() %}
        def __init__(self, *values):
            if len(values) != {{ variant.fields.len() }}:
                raise TypeError(f"Expected {{ variant.fields.len() }} arguments, found {len(values)}")
        {%- for field in variant.fields %}
            if not isinstance(values[{{ loop.index0 }}], {{ field.ty.type_name }}):
                raise TypeError(f"unexpected type for tuple element {{ loop.index0 }} - expected '{{ field.ty.type_name }}', got '{type(values[{{ loop.index0 }}])}'")
        {%- endfor %}
            super().__init__(", ".join(map(repr, values)))
            self._values = values

        def __getitem__(self, index):
            return self._values[index]

    {%-     else %}
        def __init__(self{% for field in variant.fields %}, {{ field.name }}{% endfor %}):
            {%- if !variant.fields.is_empty() %}
            super().__init__(", ".join([
                {%- for field in variant.fields %}
                "{{ field.name }}={!r}".format({{ field.name }}),
                {%- endfor %}
            ]))
            {%- for field in variant.fields %}
            self.{{ field.name }} = {{ field.name }}
            {%- endfor %}
            {%- else %}
            pass
            {%- endif %}
    {%-     endif %}

        def __repr__(self):
            return "{{ type_name }}.{{ variant_type_name }}({})".format(str(self))
    {%- endif %}
    _UniffiTemp{{ type_name }}.{{ variant_type_name }} = {{ variant_type_name }} # type: ignore
    {%- endfor %}

{{ type_name }} = _UniffiTemp{{ type_name }} # type: ignore
del _UniffiTemp{{ type_name }}


class {{ e.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        variant = buf.read_i32()
        {%- for variant in e.variants %}
        if variant == {{ loop.index }}:
            return {{ type_name }}.{{ variant.name }}(
                {%- if e.is_flat %}
                {{ string_type_node.ffi_converter_name }}.read(buf),
                {%- else %}
                {%- for field in variant.fields %}
                {{ field.ty.ffi_converter_name }}.read(buf),
                {%- endfor %}
                {%- endif %}
            )
        {%- endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    @staticmethod
    def check_lower(value):
        {%- if e.variants.is_empty() %}
        pass
        {%- else %}
        {%- for variant in e.variants %}
        if isinstance(value, {{ type_name }}.{{ variant.name }}):
            {%- for field in variant.fields %}
            {%-     if variant.has_unnamed_fields() %}
            {{ field.ty.ffi_converter_name }}.check_lower(value._values[{{ loop.index0 }}])
            {%-     else %}
            {{ field.ty.ffi_converter_name }}.check_lower(value.{{ field.name }})
            {%-     endif %}
            {%- endfor %}
            return
        {%- endfor %}
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- for variant in e.variants %}
        if isinstance(value, {{ type_name }}.{{ variant.name }}):
            buf.write_i32({{ loop.index }})
            {%- for field in variant.fields %}
            {%-     if variant.has_unnamed_fields() %}
            {{ field.ty.ffi_converter_name }}.write(value._values[{{ loop.index0 }}], buf)
            {%-     else %}
            {{ field.ty.ffi_converter_name }}.write(value.{{ field.name }}, buf)
            {%-     endif %}
            {%- endfor %}
        {%- endfor %}
