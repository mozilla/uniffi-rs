# This file was autogenerated by some hot garbage in the `uniffi` crate.
# Trust me, you don't want to mess with it!

# Common helper code.
#
# Ideally this would live in a separate .py file where it can be unittested etc
# in isolation, and perhaps even published as a re-useable package.
#
# However, it's important that the detils of how this helper code works (e.g. the
# way that different builtin types are passed across the FFI) exactly match what's
# expected by the rust code on the other side of the interface. In practice right
# now that means coming from the exact some version of `uniffi` that was used to
# compile the rust component. The easiest way to ensure this is to bundle the Python
# helpers directly inline like we're doing here.

import sys
import ctypes
import enum
import struct
import contextlib

{% include "RustBufferTemplate.py" %}

{% include "RustBufferHelper.py" %}

{% include "NamespaceLibraryTemplate.py" %}

# Public interface members begin here.

{% for e in ci.iter_enum_definitions() %}
{% include "EnumTemplate.py" %}
{%- endfor -%}

{%- for rec in ci.iter_record_definitions() %}
{% include "RecordTemplate.py" %}
{% endfor %}

{% for func in ci.iter_function_definitions() %}
{% include "TopLevelFunctionTemplate.py" %}
{% endfor %}

{% for obj in ci.iter_object_definitions() %}
{% include "ObjectTemplate.py" %}
{% endfor %}

__all__ = [
    {%- for e in ci.iter_enum_definitions() %}
    "{{ e.name()|class_name_py }}",
    {%- endfor %}
    {%- for record in ci.iter_record_definitions() %}
    "{{ record.name()|class_name_py }}",
    {%- endfor %}
    {%- for func in ci.iter_function_definitions() %}
    "{{ func.name()|fn_name_py }}",
    {%- endfor %}
    {%- for obj in ci.iter_object_definitions() %}
    "{{ obj.name()|class_name_py }}",
    {%- endfor %}
]

{% import "macros.py" as py %}