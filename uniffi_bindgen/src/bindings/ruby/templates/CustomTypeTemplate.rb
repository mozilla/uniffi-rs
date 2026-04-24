{%- match config.custom_types.get(name.as_str()) %}
{%- when None %}
# Custom type `{{ name }}` - no binding config, backed by builtin `{{ self::canonical_name(builtin) }}`.
# Values crosss the FFI as the builtin type unchanged.

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
{%- endmatch %}
