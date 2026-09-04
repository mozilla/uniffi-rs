{# Template to generate standard Rust trait method implementations for a Ruby classes.
# Expects `trait_methods` to be a bound in the including template's scope.
# (e.g. `{%- let trait_methods = obj.uniffi_trait_methods() %}`)
#}

{%- if let Some(display_fmt) = trait_methods.display_fmt %}
# The Rust `Display::fmt` implementation.
def to_s
  result = ::{{ self.module_name() }}.rust_call(
    :{{ display_fmt.ffi_func().name() }},
    {{ display_fmt|lower_method_self_rb(self) }}
  )
  {{ self.lift_rb("result", display_fmt.return_type().unwrap())? }}
end
{%- endif %}

{%- if let Some(debug_fmt) = trait_methods.debug_fmt %}
# The Rust `Debug::fmt` implementation.
def inspect
  result = ::{{ self.module_name() }}.rust_call(
    :{{ debug_fmt.ffi_func().name() }},
    {{ debug_fmt|lower_method_self_rb(self) }}
  )
  {{ self.lift_rb("result", debug_fmt.return_type().unwrap())? }}
end
{%- endif %}

{%- if let Some(eq) = trait_methods.eq_eq %}
# The Rust `Eq::eq` implementation.
def ==(other)
  return false unless other.is_a?(self.class)
  result = ::{{ self.module_name() }}.rust_call(
    :{{ eq.ffi_func().name() }},
    {{ eq|lower_method_self_rb(self) }},
    {{ self.lower_rb("other", eq.arguments()[0].as_type().borrow())? }}
  )
  {{ self.lift_rb("result", eq.return_type().unwrap())? }}
end
{%- endif %}

{%- if let Some(hash) = trait_methods.hash_hash %}
# The Rust `Hash::hash` implementation.
def hash
  result = ::{{ self.module_name() }}.rust_call(
    :{{ hash.ffi_func().name() }},
    {{ hash|lower_method_self_rb(self) }}
  )
  {{ self.lift_rb("result", hash.return_type().unwrap())? }}
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
  result = ::{{ self.module_name() }}.rust_call(
    :{{ cmp.ffi_func().name() }},
    {{ cmp|lower_method_self_rb(self) }},
    {{ self.lower_rb("other", cmp.arguments()[0].as_type().borrow())? }}
  )
  {{ self.lift_rb("result", cmp.return_type().unwrap())? }}
rescue
  nil
end
{%- endif %}
