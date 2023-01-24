
# {{func.name()}}
# Place for function description
# this inline documentation is to be run with YARD for Ruby
{% for arg in func.arguments() -%}
# @param {{ arg.name() }} [ .. ] description
{% endfor %} 
# @return [FunctionReturnValue] return field description