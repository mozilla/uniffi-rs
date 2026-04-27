{# Template to generate standard Rust trait method implementations for a Ruby classes.
# Expects `trait_methods` to be a bound in the including template's scope.
# (e.g. `{%- let trait_methods = obj.uniffi_trait_methods() %}`)
#}

{%- if let Some(display_fmt) = trait_methods.display_fmt %}
# The Rust `Display::fmt` implementation.
def to_s
  result = ::{{ ci.namespace()|class_name_rb }}.rust_call(
    :{{ display_fmt.ffi_func().name() }},
    {{ display_fmt|lower_method_self_rb(config) }}
  )
  {{ "result"|lift_rb(display_fmt.return_type().unwrap(), config) }}
end
{%- endif %}

{%- if let Some(debug_fmt) = trait_methods.debug_fmt %}
# The Rust `Debug::fmt` implementation.
def inspect
  result = ::{{ ci.namespace()|class_name_rb }}.rust_call(
    :{{ debug_fmt.ffi_func().name() }},
    {{ debug_fmt|lower_method_self_rb(config) }}
  )
  {{ "result"|lift_rb(debug_fmt.return_type().unwrap(), config) }}
end
{%- endif %}

{%- if let Some(eq) = trait_methods.eq_eq %}
# The Rust `Eq::eq` implementation.
def ==(other)
  return false unless other.is_a?(self.class)
  result = ::{{ ci.namespace()|class_name_rb }}.rust_call(
    :{{ eq.ffi_func().name() }},
    {{ eq|lower_method_self_rb(config) }},
    {{ "other"|lower_rb(eq.arguments()[0].as_type().borrow(), config) }}
  )
  {{ "result"|lift_rb(eq.return_type().unwrap(), config) }}
end
{%- endif %}

{%- if let Some(hash) = trait_methods.hash_hash %}
# The Rust `Hash::hash` implementation.
def hash
  result = ::{{ ci.namespace()|class_name_rb }}.rust_call(
    :{{ hash.ffi_func().name() }},
    {{ hash|lower_method_self_rb(config) }}
  )
  {{ "result"|lift_rb(hash.return_type().unwrap(), config) }}
end

def eql?(other)
  self == other
end
{%- endif %}

{%- if let Some(cmp) = trait_methods.ord_cmp %}
# The Rust `Ord::cmp` implementation.
include Comparable

def <=>(other)
  # do we need this?
  # return nil unless other.is_a?(self.class)
  result = ::{{ ci.namespace()|class_name_rb }}.rust_call(
    :{{ cmp.ffi_func().name() }},
    {{ cmp|lower_method_self_rb(config) }},
    {{ "other"|lower_rb(cmp.arguments()[0].as_type().borrow(), config) }}
  )
  {{ "result"|lift_rb(cmp.return_type().unwrap(), config) }}
rescue
  nil
end
{%- endif %}
