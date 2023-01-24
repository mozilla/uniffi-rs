
/**
 * fun name: {{func.name()}}
 * Some fun description text. This is supposed to be run with KDoc or Dokka.
{% for arg in func.arguments() -%}
 * @param[{{ arg.name() }}] description.
{% endfor %} 
 * @return something something.
 */