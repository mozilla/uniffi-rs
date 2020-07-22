class {{ e.name() }}(enum.Enum):
    {% for value in e.values() -%}
    {{ value|enum_name_py }} = {{ loop.index }}
    {% endfor -%}