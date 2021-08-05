{%- for e in ci.iter_error_definitions() %}
pub enum {{ e.name()|class_name_rs }} {
  {%- for variant in e.variants() %}
  {{ variant.name()|class_name_rs }} {%- if variant.has_fields() %}{ {% for field in variant.fields() %}{{ field.name()|var_name_rs }}: {{field.type_()|type_rs}}{% if !loop.last %}, {% endif %}{% endfor %} }{% endif %},
  {%- endfor %}
}
{%- endfor %}
