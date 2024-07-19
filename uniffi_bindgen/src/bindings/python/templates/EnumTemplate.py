{#
# Python has a built-in `enum` module which is nice to use, but doesn't support
# variants with associated data. So, we switch here, and generate a stdlib `enum`
# when none of the variants have associated data, or a generic nested-class
# construct when they do.
#}
{% if e.is_flat() %}

class {{ type_name }}(enum.Enum):
    {%- call py::docstring(e, 4) %}
    {%- for variant in e.variants() %}
    {{ variant.name() }} = {{ e|variant_discr_literal(loop.index0) }}
    {%- call py::docstring(variant, 4) %}
    {% endfor %}
{% else %}

class {{ type_name }}:
    {%- call py::docstring(e, 4) %}
    def __init__(self):
        raise RuntimeError("{{ type_name }} cannot be instantiated directly")

    # Each enum variant is a nested class of the enum itself.
    {% for variant in e.variants() -%}
    class {{ variant.name() }}:
        {%- call py::docstring(variant, 8) %}

    {%-  if variant.has_nameless_fields() %}
        def __init__(self, *values):
            if len(values) != {{ variant.fields().len() }}:
                raise TypeError(f"Expected {{ variant.fields().len() }} arguments, found {len(values)}")
        {%- for field in variant.fields() %}
            if not isinstance(values[{{ loop.index0 }}], {{ field|type_name }}):
                raise TypeError(f"unexpected type for tuple element {{ loop.index0 }} - expected '{{ field|type_name }}', got '{type(values[{{ loop.index0 }}])}'")
        {%- endfor %}
            self._values = values

        def __getitem__(self, index):
            return self._values[index]

        def __str__(self):
            return f"{{ type_name }}.{{ variant.name() }}{self._values!r}"

        def __eq__(self, other):
            if not other.is_{{ variant.name() }}():
                return False
            return self._values == other._values

    {%-  else -%}
        {%- for field in variant.fields() %}
        {{ field.name() }}: "{{ field|type_name }}"
        {%- call py::docstring(field, 8) %}
        {%- endfor %}

        def __init__(self,{% for field in variant.fields() %}{{ field.name() }}: "{{- field|type_name }}"{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
            {%- if variant.has_fields() %}
            {%- for field in variant.fields() %}
            self.{{ field.name() }} = {{ field.name() }}
            {%- endfor %}
            {%- else %}
            pass
            {%- endif %}

        def __str__(self):
            return "{{ type_name }}.{{ variant.name() }}({% for field in variant.fields() %}{{ field.name()  }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in variant.fields() %}self.{{ field.name()  }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

        def __eq__(self, other):
            if not other.is_{{ variant.name() }}():
                return False
            {%- for field in variant.fields() %}
            if self.{{ field.name() }} != other.{{ field.name() }}:
                return False
            {%- endfor %}
            return True
    {%  endif %}
    {% endfor %}

    # For each variant, we have an `is_NAME` method for easily checking
    # whether an instance is that variant.
    {% for variant in e.variants() -%}
    def is_{{ variant.name() }}(self) -> bool:
        return isinstance(self, {{ type_name }}.{{ variant.name() }})
    {% endfor %}

# Now, a little trick - we make each nested variant class be a subclass of the main
# enum class, so that method calls and instance checks etc will work intuitively.
# We might be able to do this a little more neatly with a metaclass, but this'll do.
{% for variant in e.variants() -%}
{{ type_name }}.{{ variant.name() }} = type("{{ type_name }}.{{ variant.name() }}", ({{ type_name }}.{{variant.name()}}, {{ type_name }},), {})  # type: ignore
{% endfor %}

{% endif %}

class {{ ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        variant = buf.read_i32()

        {%- for variant in e.variants() %}
        if variant == {{ loop.index }}:
            {%- if e.is_flat() %}
            return {{ type_name }}.{{variant.name()}}
            {%- else %}
            return {{ type_name }}.{{variant.name()}}(
                {%- for field in variant.fields() %}
                {{ field|read_fn }}(buf),
                {%- endfor %}
            )
            {%- endif %}
        {%- endfor %}
        raise InternalError("Raw enum value doesn't match any cases")

    @staticmethod
    def check_lower(value):
        {%- if e.variants().is_empty() %}
        pass
        {%- else %}
        {%- for variant in e.variants() %}
        {%- if e.is_flat() %}
        if value == {{ type_name }}.{{ variant.name() }}:
        {%- else %}
        if value.is_{{ variant.name() }}():
        {%- endif %}
            {%- for field in variant.fields() %}
            {%- if variant.has_nameless_fields() %}
            {{ field|check_lower_fn }}(value._values[{{ loop.index0 }}])
            {%- else %}
            {{ field|check_lower_fn }}(value.{{ field.name() }})
            {%- endif %}
            {%- endfor %}
            return
        {%- endfor %}
        raise ValueError(value)
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- for variant in e.variants() %}
        {%- if e.is_flat() %}
        if value == {{ type_name }}.{{ variant.name() }}:
            buf.write_i32({{ loop.index }})
        {%- else %}
        if value.is_{{ variant.name() }}():
            buf.write_i32({{ loop.index }})
            {%- for field in variant.fields() %}
            {%- if variant.has_nameless_fields() %}
            {{ field|write_fn }}(value._values[{{ loop.index0 }}], buf)
            {%- else %}
            {{ field|write_fn }}(value.{{ field.name() }}, buf)
            {%- endif %}
            {%- endfor %}
        {%- endif %}
        {%- endfor %}

