{#
# Python has a built-in `enum` module which is nice to use, but doesn't support
# variants with associated data. So, we switch here, and generate a stdlib `enum`
# when none of the variants have associated data, or a generic nested-class
# construct when they do.
#}
{% if e.is_flat() %}

class {{ e.name()|class_name_py }}(enum.Enum):
    {% for variant in e.variants() -%}
    {{ variant.name()|enum_name_py }} = {{ loop.index }}
    {% endfor %}

{% else %}

class {{ e.name()|class_name_py }}(object):
    def __init__(self):
        raise RuntimeError("{{ e.name()|class_name_py }} cannot be instantiated directly")

    # Each enum variant is a nested class of the enum itself.
    {% for variant in e.variants() -%}
    class {{ variant.name()|enum_name_py }}(object):
        def __init__(self,{% for field in variant.fields() %}{{ field.name()|var_name_py }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
            {% if variant.has_fields() %}
            {%- for field in variant.fields() %}
            self.{{ field.name()|var_name_py }} = {{ field.name()|var_name_py }}
            {%- endfor %}
            {% else %}
            pass
            {% endif %}

        def __str__(self):
            return "{{ e.name()|class_name_py }}.{{ variant.name()|enum_name_py }}({% for field in variant.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in variant.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

        def __eq__(self, other):
            if not other.is_{{ variant.name()|var_name_py }}():
                return False
            {%- for field in variant.fields() %}
            if self.{{ field.name()|var_name_py }} != other.{{ field.name()|var_name_py }}:
                return False
            {%- endfor %}
            return True
    {% endfor %}

    # For each variant, we have an `is_NAME` method for easily checking
    # whether an instance is that variant.
    {% for variant in e.variants() -%}
    def is_{{ variant.name()|var_name_py }}(self):
        return isinstance(self, {{ e.name()|class_name_py }}.{{ variant.name()|enum_name_py }})
    {% endfor %}

# Now, a little trick - we make each nested variant class be a subclass of the main
# enum class, so that method calls and instance checks etc will work intuitively.
# We might be able to do this a little more neatly with a metaclass, but this'll do.
{% for variant in e.variants() -%}
{{ e.name()|class_name_py }}.{{ variant.name()|enum_name_py }} = type("{{ e.name()|class_name_py }}.{{ variant.name()|enum_name_py }}", ({{ e.name()|class_name_py }}.{{variant.name()|enum_name_py}}, {{ e.name()|class_name_py }},), {})
{% endfor %}

{% endif %}
