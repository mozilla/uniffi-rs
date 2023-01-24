"""
fun name: {{func.name()}}
Some fun description text. This is supposed to be run with Sphinx
https://docutils.sourceforge.io/rst.html documentation
Some examples https://docutils.sourceforge.io/docutils/statemachine.py
{% for arg in func.arguments() -%}
:Parameter {{ arg.name() }}: arg type
{% endfor %} 
:Return:  something something.
"""