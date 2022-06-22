class {{ name }}({{ fields|comma_join}}): {{ parent }}() {
    {%- if as_string %}
    override val message: String
        get() {
            {{ as_string.body }}
        }
    {%- endif %}
}
