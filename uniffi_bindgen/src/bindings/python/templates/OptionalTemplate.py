{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class FfiConverter{{ canonical_type_name }}(FfiConverterUsingByteBuffer):
    @staticmethod
    def _write(value, buf):
        FfiConverterOptional._write(value, buf, lambda v, buf: {{ "v"|write_py("buf", inner_type) }})

    @staticmethod
    def _read(buf):
        return FfiConverterOptional._read(buf, lambda buf: {{ "buf"|read_py(inner_type) }})
