{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class FfiConverter{{ canonical_type_name }}(FfiConverterUsingByteBuffer):
    @staticmethod
    def _write(value, buf):
        FfiConverterSequence._write(value, buf, lambda v, buf: {{ "v"|write_var("buf", inner_type) }})

    @staticmethod
    def _read(buf):
        return FfiConverterSequence._read(buf, lambda buf: {{ "buf"|read_var(inner_type) }})
