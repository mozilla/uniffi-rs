{#
# Python has a built-in `enum` module which is nice to use, but doesn't support
# variants with associated data. So, we switch here, and generate a stdlib `enum`
# when none of the variants have associated data, or a generic nested-class
# construct when they do.
#}
{%- let type_name = e.self_type.type_name %}
{%- let uniffi_trait_methods = e.uniffi_trait_methods %}

{% if e.is_flat %}

class {{ type_name }}(enum.Enum):
    {{ e.docstring|docstring(4) -}}
    {%- for variant in e.variants %}
    {{ variant.name }} = {{ variant.discr.py_lit }}
    {{ variant.docstring|docstring(4) -}}
    {% endfor %}

    {%- for meth in e.methods -%}
    {%- let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(self, {% include "CallableArgs.py" %}) -> {{ callable.return_type.type_name }}:
        {{ meth.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
    {%- endfor %}
{% else %}

class {{ type_name }}:
    {{ e.docstring|docstring(4) -}}
    def __init__(self):
        raise RuntimeError("{{ type_name }} cannot be instantiated directly")

    # Each enum variant is a nested class of the enum itself.
    {% for variant in e.variants -%}
    @dataclass
    class {{ variant.name }}:
        {{ variant.docstring|docstring(8) -}}

    {%-  if variant.has_unnamed_fields() %}
        def __init__(self, *values):
            if len(values) != {{ variant.fields.len() }}:
                raise TypeError(f"Expected {{ variant.fields.len() }} arguments, found {len(values)}")
            self._values = values

        def __getitem__(self, index):
            return self._values[index]

    {% filter indent(8) %}
    {% include "UniffiTraitImpls.py" %}
    {% endfilter %}

    {# "builtin" methods not handled by a uniffi_trait #}
    {%-     if uniffi_trait_methods.display_fmt.is_none() %}
        def __str__(self):
            return f"{{ type_name }}.{{ variant.name }}{self._values!r}"
    {%-     endif %}

    {%-     if uniffi_trait_methods.eq_eq.is_none() %}
        def __eq__(self, other):
            if not other.is_{{ variant.name }}():
                return False
            return self._values == other._values
    {%-     endif %}

    {%-  else %}
        def __init__(self, {% for field in variant.fields %}
        {{- field.name }}: {{- field.ty.type_name}}
        {%- if let Some(default) = field.default %} = {{ default.arg_literal }}{% endif %}
        {%- if !loop.last %}, {% endif %}
        {%- endfor %}):
        {%- for field in variant.fields %}
            {%- match field.default %}
            {%- when None %}
            self.{{ field.name }} = {{ field.name }}
            {%- when Some(default) %}
            {%- if default.is_arg_literal %}
            self.{{ field.name }} = {{ field.name }}
            {%- else %}
            if {{ field.name }} is {{ default.arg_literal }}:
                self.{{ field.name }} = {{ default.py_default }}
            else:
                self.{{ field.name }} = {{ field.name }}
            {%- endif %}
            {%- endmatch %}
            {# not sure what to do with docstrings - see #2552, suggests putting them after the field init #}
            {{ field.docstring|docstring(8) -}}
        {%- endfor %}
            pass

    {% filter indent(8) %}
    {%      include "UniffiTraitImpls.py" %}
    {% endfilter %}
    {# "builtin" methods not handled by a uniffi_trait #}
    {%-     if uniffi_trait_methods.display_fmt.is_none() %}
        def __str__(self):
            return "{{ type_name }}.{{ variant.name }}({% for field in variant.fields %}{{ field.name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in variant.fields %}self.{{ field.name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})
    {%-     endif %}
    {%-     if uniffi_trait_methods.eq_eq.is_none() %}
        def __eq__(self, other):
            if not other.is_{{ variant.name }}():
                return False
            {%- for field in variant.fields %}
            if self.{{ field.name }} != other.{{ field.name }}:
                return False
            {%- endfor %}
            return True
    {%-     endif %}
    {%-  endif %}

    {% endfor %}

    # For each variant, we have `is_NAME` and `is_name` methods for easily checking
    # whether an instance is that variant.
    {% for variant in e.variants -%}
    def is_{{ variant.name }}(self) -> bool:
        return isinstance(self, {{ type_name }}.{{ variant.name }})

    {#- We used to think we used `is_NAME` but did `is_name` instead. In #2270 we decided to do both. #}
    {%- if variant.name != variant.name.to_snake_case() %}
    def is_{{ variant.name.to_snake_case() }}(self) -> bool:
        return isinstance(self, {{ type_name }}.{{ variant.name }})
    {%- endif %}
    {% endfor %}

    {%- for meth in e.methods -%}
    {%- let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(self, {% include "CallableArgs.py" %}) -> {{ callable.return_type.type_name }}:
        {{ meth.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
    {%- endfor %}

# Now, a little trick - we make each nested variant class be a subclass of the main
# enum class, so that method calls and instance checks etc will work intuitively.
# We might be able to do this a little more neatly with a metaclass, but this'll do.
{% for variant in e.variants -%}
{{ type_name }}.{{ variant.name }} = type("{{ type_name }}.{{ variant.name }}", ({{ type_name }}.{{variant.name}}, {{ type_name }},), {})  # type: ignore
{% endfor %}

{% endif %}

class {{ e.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        variant = buf.read_i32()

        {%- for variant in e.variants %}
        if variant == {{ loop.index }}:
            {%- if e.is_flat %}
            return {{ type_name }}.{{variant.name}}
            {%- else %}
            return {{ type_name }}.{{variant.name}}(
                {%- for field in variant.fields %}
                {{ field.ty.ffi_converter_name }}.read(buf),
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
        {%- if e.is_flat %}
        if value == {{ type_name }}.{{ variant.name }}:
        {%- else %}
        if value.is_{{ variant.name }}():
        {%- endif %}
            {%- for field in variant.fields %}
            {%- if variant.has_unnamed_fields() %}
            {{ field.ty.ffi_converter_name }}.check_lower(value._values[{{ loop.index0 }}])
            {%- else %}
            {{ field.ty.ffi_converter_name }}.check_lower(value.{{ field.name }})
            {%- endif %}
            {%- endfor %}
            return
        {%- endfor %}
        raise ValueError(value)
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- for variant in e.variants %}
        {%- if e.is_flat %}
        if value == {{ type_name }}.{{ variant.name }}:
            buf.write_i32({{ loop.index }})
        {%- else %}
        if value.is_{{ variant.name }}():
            buf.write_i32({{ loop.index }})
            {%- for field in variant.fields %}
            {%- if variant.has_unnamed_fields() %}
            {{ field.ty.ffi_converter_name }}.write(value._values[{{ loop.index0 }}], buf)
            {%- else %}
            {{ field.ty.ffi_converter_name }}.write(value.{{ field.name }}, buf)
            {%- endif %}
            {%- endfor %}
        {%- endif %}
        {%- endfor %}

