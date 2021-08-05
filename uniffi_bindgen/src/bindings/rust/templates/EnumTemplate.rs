{% if e.is_flat() %}

pub enum {{ e.name()|class_name_rs }} {
  {% for variant in e.variants() -%}
  {{ variant.name()|enum_name_rs }} = {{ loop.index }},
  {% endfor %}
}

{% else %}

pub enum {{ e.name()|class_name_rs }} {
  {% for variant in e.variants() -%}
  {{ variant.name()|enum_name_rs }} {% if variant.has_fields() %} { {%- call rs::arg_list_decl(variant.fields()) -%} } {% endif %},
  {% endfor %}
}

{% endif %}
