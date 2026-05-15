{%- let cbi = ci.get_callback_interface_definition(name).unwrap() %}
{%- let cbi_name = name %}
{%- let interface_name = cbi.name() %}

class {{ interface_name }}
  {% for meth in cbi.methods() -%}
  def {{ meth.name()|fn_name_rb }}(**_args)
    raise NoMethodError, 'method should be implemented in concrete class'
  end
  {% endfor %}
end

# The FfiConverter for the {{ name }} callback interface.
{{ self::canonical_name(cbi.as_type().borrow()) }}FfiConverter = CallbackInterfaceFfiConverter.new

{% include "CallbackInterfaceImpl.rb" %}
