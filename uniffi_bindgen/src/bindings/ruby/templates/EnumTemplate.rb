{% if e.is_flat() %}

class {{ e.name()|class_name_rb }}
  {% for variant in e.variants() -%}
  {{ variant.name()|enum_name_rb }} = {{ e|variant_discr_literal(loop.index0) }}
  {% endfor %}
end

{% else %}

class {{ e.name()|class_name_rb }}
  def initialize
    raise RuntimeError, '{{ e.name()|class_name_rb }} cannot be instantiated directly'
  end

  # Each enum variant is a nested class of the enum itself.
  {% for variant in e.variants() -%}
  class {{ variant.name()|enum_name_rb }}
    {%- let named_fields = variant.has_fields() && !variant.fields()[0].name().is_empty() %}
    {% if variant.has_fields() %}
    {%- if named_fields %}
      attr_reader {% for field in variant.fields() %}:{{ field.name()|var_name_rb }}{% if loop.last %}{% else %}, {% endif %}{%- endfor %}
    {%- else %}
      attr_reader :values
    {%- endif %}
    {% endif %}
    {%- if named_fields %}
    def initialize({% for field in variant.fields() %}{{ field.name()|var_name_rb -}}:
      {%- match field.default_value() %}
      {%- when Some(default) %} {{ field|field_default_rb }}
      {%- else %}
      {% endmatch %}
      {%- if loop.last %}{% else %}, {% endif -%}{% endfor %})
        {%- for field in variant.fields() %}
        @{{ field.name()|var_name_rb }} = {{ field.name()|var_name_rb }}
        {%- endfor %}
    end
    {%- else %}
    def initialize({% for field in variant.fields() %}v{{ loop.index }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})
      {% if variant.has_fields() %}
        @values = [{% for field in variant.fields() %}v{{ loop.index }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}]
      {% else %}
      {% endif %}
    end

    def [](index)
      @values[index]
    end
    {% endif %}

    def to_s
      "{{ e.name()|class_name_rb }}::{{ variant.name()|enum_name_rb }}"
    end

    def ==(other)
      return false unless other.respond_to?(:{{variant.name()|var_name_rb}}?)
      return false unless other.{{ variant.name()|var_name_rb }}?
      {%- if named_fields %}
      {%- for field in variant.fields() %}
        return false if @{{ field.name()|var_name_rb }} != other.{{ field.name()|var_name_rb }}
      {%- endfor %}
      {%- else %}
      {%- if variant.has_fields() %}
        return false if @values != other.values
      {% endif %}
      {% endif %}
      true
    end

    # For each variant, we have an `NAME?` method for easily checking
    # whether an instance is that variant.
    {% for variant in e.variants() %}
    def {{ variant.name()|var_name_rb }}?
      instance_of? {{ e.name()|class_name_rb }}::{{ variant.name()|enum_name_rb }}
    end
    {% endfor %}
  end
  {% endfor %}
end

{% endif %}
