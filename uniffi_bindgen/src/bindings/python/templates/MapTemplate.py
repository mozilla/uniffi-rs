{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let canonical_type_name = outer_type|canonical_name %}

class FfiConverter{{ canonical_type_name }}(FfiConverterUsingByteBuffer):
    @staticmethod
    def _write(value, buf):
        def inner_write(key, value, buf):
            {{ "key"|write_var("buf", Type::String) }}
            {{ "value"|write_var("buf", inner_type) }}

        FfiConverterDictionary._write(value, buf, inner_write)

    @staticmethod
    def _read(buf):
        def inner_read(buf):
            key = {{ "buf"|read_var(TypeIdentifier::String) }}
            value = {{ "buf"|read_var(inner_type) }}
            return (key, value)

        return FfiConverterDictionary._read(buf, inner_read)
