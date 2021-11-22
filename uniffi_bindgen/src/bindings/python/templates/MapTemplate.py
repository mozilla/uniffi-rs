{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class FfiConverter{{ canonical_type_name }}(FfiConverterUsingByteBuffer):
    @staticmethod
    def _write(value, buf):
        def inner_write(key, value, buf):
            {{ "key"|write_py("buf", Type::String) }}
            {{ "value"|write_py("buf", inner_type) }}

        FfiConverterDictionary._write(value, buf, inner_write)

    @staticmethod
    def _read(buf):
        def inner_read(buf):
            key = {{ "buf"|read_py(TypeIdentifier::String) }}
            value = {{ "buf"|read_py(inner_type) }}
            return (key, value)

        return FfiConverterDictionary._read(buf, inner_read)
