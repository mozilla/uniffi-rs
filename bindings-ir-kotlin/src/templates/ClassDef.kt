{{ vis }} class {{ name }} internal constructor({{ fields|comma_join }}) {
    {%- if constructor %}
    {{ constructor.vis }} constructor({{ constructor.args|comma_join }}): this({{ constructor.initializers|comma_join }})
    {%- endif %}

    {%- for method in methods %}
    {%- if method.method_type.ir_type != "Static" %}
    {{ method.vis }} fun {{ method.name }}(
        {%- for arg in method.args %}
        {{ arg }},
        {%- endfor %}
    ){%- if method.return_type %}: {{ method.return_type }}{%- endif %} {
        {{ method.body }}
    }
    {%- endif %}
    {%- endfor %}

    {%- if destructor %}
    val shouldRunDestructor = java.util.concurrent.atomic.AtomicBoolean(true)
    class Cleaner (
        {%- for field in fields %}
        {{ field }},
        {%- endfor %}
        val shouldRunDestructor: java.util.concurrent.atomic.AtomicBoolean
    ) : Runnable {
        override fun run() {
            if (shouldRunDestructor.get()) {
                {{ destructor.body }}
            }
        }
    }
    {%- endif %}

    {%- if into_rust %}
    fun intoRust(): {{ into_rust.return_type }} {
        {%- if destructor %}
        shouldRunDestructor.set(false)
        {%- endif %}
        {{ into_rust.body }}
    }
    {%- endif %}

    companion object {
        {%- for method in methods %}
        {%- if method.method_type.ir_type == "Static" %}
        {{ method.vis }} fun {{ method.name }}(
            {%- for arg in method.args %}
            {{ arg }},
            {%- endfor %}
        ){%- if method.return_type %}: {{ method.return_type }}{%- endif %} {
            {{ method.body }}
        }
        {%- endif %}
        {%- endfor %}
    }
}
