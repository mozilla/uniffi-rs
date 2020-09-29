class {{ e.name()|class_name_py }}(enum.Enum):
    {% for variant in e.variants() -%}
    {{ variant|enum_name_py }} = {{ loop.index }}
    {% endfor %}
