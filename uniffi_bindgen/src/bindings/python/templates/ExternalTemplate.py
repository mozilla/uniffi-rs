{%- let name = self.name() %}
{%- let crate_name = self.crate_name() %}

class FfiConverter{{ name }}(FfiConverterUsingByteBuffer):
    @staticmethod
    def _write(value, buf):
        from {{ crate_name|fn_name_py }} import {{ name }};
        {{ name }}._write(value, buf)

    @staticmethod
    def _read(buf):
        from {{ crate_name|fn_name_py }} import {{ name }};
        return {{ name }}._read(buf)
