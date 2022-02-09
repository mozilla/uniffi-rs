{%- let name = self.name() %}

from {{ self.crate_name()|fn_name }} import FfiConverterType{{ name }}
