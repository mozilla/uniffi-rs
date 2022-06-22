{{ vis }} fun {{ name }}({{ args|comma_join }}){% if return_type %} : {{ return_type }}{%- endif %} {
    {{ body }}
}
