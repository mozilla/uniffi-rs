class {{ e.name() }}(enum.Enum):
    {% for variant in e.variants() -%}
    {{ variant|enum_name_py }} = {{ loop.index }}
    {% endfor -%}