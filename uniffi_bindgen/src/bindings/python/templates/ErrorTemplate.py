# {{ e.name }}
# We want to define each variant as a nested class that's also a subclass,
# which is tricky in Python.  To accomplish this we're going to create each
# class separately, then manually add the child classes to the base class's
# __dict__.  All of this happens in dummy class to avoid polluting the module
# namespace.
class {{ e.name }}(Exception):
    {{ e.docstring|docindent(4) }}
    pass

_UniffiTemp{{ e.name }} = {{ e.name }}

class {{ e.name }}:  # type: ignore
    {%- for variant in e.variants -%}
    {%- if e.is_flat() %}
    class {{ variant.name }}(_UniffiTemp{{ e.name }}):
        {{ variant.docstring|docindent(8) -}}
        def __repr__(self):
            return "{{ e.name }}.{{ variant.name }}({})".format(repr(str(self)))
    {%- else %}
    class {{ variant.name }}(_UniffiTemp{{ e.name }}):
        {{ variant.docstring|docindent(8) -}}
    {%-     if variant.has_nameless_fields() %}
        def __init__(self, *values):
            if len(values) != {{ variant.fields.len() }}:
                raise TypeError(f"Expected {{ variant.fields.len() }} arguments, found {len(values)}")
        {%- for field in variant.fields %}
            if not isinstance(values[{{ loop.index0 }}], {{ field.ty.type_name }}):
                raise TypeError(f"unexpected type for tuple element {{ loop.index0 }} - expected '{{ field.name }}', got '{type(values[{{ loop.index0 }}])}'")
        {%- endfor %}
            super().__init__(", ".join(map(repr, values)))
            self._values = values

        def __getitem__(self, index):
            return self._values[index]

    {%-     else %}
        def __init__(self{% for field in variant.fields %}, {{ field.name }}{% endfor %}):
            {%- if variant.has_fields() %}
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
            return "{{ e.name }}.{{ variant.name }}({})".format(str(self))
    {%- endif %}
    _UniffiTemp{{ e.name }}.{{ variant.name }} = {{ variant.name }} # type: ignore
    {%- endfor %}

{{ e.name }} = _UniffiTemp{{ e.name }} # type: ignore
del _UniffiTemp{{ e.name }}


class {{ ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        variant = buf.read_i32()
        {%- for variant in e.variants %}
        if variant == {{ loop.index }}:
            {%- if e.is_flat() %}
            return {{ e.name }}.{{ variant.name }}(
                {{ globals.string_type|read_fn }}(buf)
            )
            {%- else %}
            return {{ e.name }}.{{ variant.name }}(
                {%- for field in variant.fields %}
                {{ field|read_fn }}(buf),
                {%- endfor %}
            )
            {%- endif %}
        {%- endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    @staticmethod
    def check_lower(value):
        {%- if e.variants.is_empty() %}
        pass
        {%- else %}
        {%- for variant in e.variants %}
        if isinstance(value, {{ e.name }}.{{ variant.name }}):
            {%- for field in variant.fields %}
            {%-     if variant.has_nameless_fields() %}
            {{ field|check_lower_fn }}(value._values[{{ loop.index0 }}])
            {%-     else %}
            {{ field|check_lower_fn }}(value.{{ field.name }})
            {%-     endif %}
            {%- endfor %}
            return
        {%- endfor %}
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- for variant in e.variants %}
        if isinstance(value, {{ e.name }}.{{ variant.name }}):
            buf.write_i32({{ loop.index }})
            {%- for field in variant.fields %}
            {%- if variant.has_nameless_fields() %}
            {{ field|write_fn }}(value._values[{{ loop.index0 }}], buf)
            {%- else %}
            {{ field|write_fn }}(value.{{ field.name }}, buf)
            {%- endif %}
            {%- endfor %}
        {%- endfor %}
