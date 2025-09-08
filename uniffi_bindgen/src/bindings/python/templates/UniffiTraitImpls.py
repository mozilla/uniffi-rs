{# expects var `uniffi_trait_methods` #}
{%- if let Some(fmt) = uniffi_trait_methods.debug_fmt %}
{%-    let callable = fmt.callable %}
# The Rust `Debug::fmt` implementation.
def __repr__(self) -> {{ callable.return_type.type_name }}:
    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter -%}
{%- endif %}

{%- if let Some(fmt) = uniffi_trait_methods.display_fmt %}
{%-    let callable = fmt.callable %}
# The Rust `Display::fmt` implementation.
def __str__(self) -> {{ callable.return_type.type_name }}:
    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter -%}
{%- endif %}

{%- if let Some(eq) = uniffi_trait_methods.eq_eq %}
{%-    let callable = eq.callable %}
# The Rust `Eq::eq` implementation.
def __eq__(self, other: object) -> {{ callable.return_type.type_name }}:
    if not isinstance(other, {{ callable.self_type().unwrap().type_name }}):
        return NotImplemented

    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter -%}
{%- endif %}
{%- if let Some(ne) = uniffi_trait_methods.eq_ne %}
{%-    let callable = ne.callable %}
# The Rust `Eq::ne` implementation.
def __ne__(self, other: object) -> {{ callable.return_type.type_name }}:
    if not isinstance(other, {{ callable.self_type().unwrap().type_name }}):
        return NotImplemented
    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter -%}
{%- endif %}

{%- if let Some(hash) = uniffi_trait_methods.hash_hash %}
{%-    let callable = hash.callable %}
# The Rust `Hash::hash` implementation.
def __hash__(self) -> {{ callable.return_type.type_name }}:
    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter -%}

{%- endif %}

{%- if let Some(cmp) = uniffi_trait_methods.ord_cmp %}
{%-    let callable = cmp.callable %}
# The Rust `Ord::cmp` implementation.
# lol/sob, python3 got rid of the perfect python2 `.__cmp__()` :(
def __rust_cmp__(self, other) -> {{ callable.return_type.type_name }}:
    {% filter indent(4) -%}
    {% include "CallableBody.py" -%}
    {% endfilter %}

def __lt__(self, other) -> bool:
    return self.__rust_cmp__(other) < 0

def __le__(self, other) -> bool:
    return self.__rust_cmp__(other) <= 0

def __gt__(self, other) -> bool:
    return self.__rust_cmp__(other) > 0

def __ge__(self, other) -> bool:
    return self.__rust_cmp__(other) >= 0
{%- endif %}
