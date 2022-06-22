private val objectCleaner = java.lang.ref.Cleaner.create()
public class InternalException(message: String) : RuntimeException(message)
{%- if native_library %}
{{ native_library }}
{%- endif %}

{%- for name, definition in definitions %}
{{ definition }}
{%- endfor %}

{%- if tests %}
// Define tests
{%- for test in tests %}
{{ test }}
{%- endfor %}

// Run tests
{%- for test in tests %}
{{ test.name }}()
{%- endfor %}
{%- endif %}
