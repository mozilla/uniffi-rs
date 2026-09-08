{%- let canonical_type_name = self::canonical_name(type_) %}
{%- match config.custom_types.get(name.as_str()) %}
{%- when None %}
# Custom type `{{ name }}` - no binding config, backed by builtin `{{ self::canonical_name(builtin) }}`.
# Identity lower coerces the builtin (`uniffi_in_range` / `uniffi_utf8` / …)
# so imported primitive FFI args still run `to_int` / `to_str` here.

def self.uniffi_lift_{{ canonical_type_name }}(raw)
  raw
end

def self.uniffi_lower_{{ canonical_type_name }}(v)
  {{ self.coerce_rb("v", builtin)? }}
end

def self.uniffi_check_lower_{{ canonical_type_name }}(v)
  {{ self.check_lower_rb("v", builtin)? }}
end

{%- when Some(cfg) %}
# Custom type `{{ name }}` - binding config supplied, backed by builtin `{{ self::canonical_name(builtin) }}`.
{%- if cfg.has_conversion() %}
#   lift expression: {{ cfg.lift("raw_value") }}
#   lower expression: {{ cfg.lower("custom_value") }}
{%- endif %}
{%- match cfg.imports %}
{%- when Some(imports) %}
{%- for import_name in imports %}
require '{{ import_name }}'
{%- endfor %}
{%- when None %}
{%- endmatch %}

def self.uniffi_lift_{{ canonical_type_name }}(raw)
{%- if cfg.has_conversion() %}
  {{ cfg.lift("raw") }}
{%- else %}
  raw
{%- endif %}
end

def self.uniffi_lower_{{ canonical_type_name }}(v)
{%- if cfg.has_conversion() %}
  {{ cfg.lower("v") }}
{%- else %}
{%- match cfg.type_name %}
{%- when Some(_) %}
  v
{%- else %}
  {{ self.coerce_rb("v", builtin)? }}
{%- endmatch %}
{%- endif %}
end

def self.uniffi_check_lower_{{ canonical_type_name }}(v)
{%- match cfg.type_name %}
{%- when Some(type_name) %}
  raise TypeError, "Expected {{ type_name }}, got #{v.class}" unless v.is_a?({{ type_name }})
{%- else %}
{%- endmatch %}
end
{%- endmatch %}
