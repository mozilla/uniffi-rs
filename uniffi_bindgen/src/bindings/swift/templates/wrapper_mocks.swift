{%- import "macros.swift" as swift %}

import Foundation

{% for type_ in ci.iter_types() %}
{%- let type_name = type_|type_name %}
{%- let ffi_converter_name = type_|ffi_converter_name %}
{%- let canonical_type_name = type_|canonical_name %}
{%- let contains_object_references = ci.item_contains_object_references(type_) %}

{%- match type_ %}

{%- when Type::Object{ name, module_path, imp } %}
{%- include "MockTemplate.swift" %}

{%- else %}
{%- endmatch %}
{%- endfor %}
