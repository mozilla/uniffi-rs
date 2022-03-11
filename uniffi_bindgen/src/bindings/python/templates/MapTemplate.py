{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let key_ffi_converter = Type::String.borrow()|ffi_converter_name %}
{%- let value_ffi_converter = inner_type|ffi_converter_name %}

class {{ outer_type|ffi_converter_name }}(FfiConverterRustBuffer):
    @classmethod
    def write(cls, items, buf):
        buf.writeI32(len(items))
        for (key, value) in items.items():
            {{ key_ffi_converter }}.write(key, buf)
            {{ value_ffi_converter }}.write(value, buf)

    @classmethod
    def read(cls, buf):
        count = buf.readI32()
        if count < 0:
            raise InternalError("Unexpected negative map size")
        return {
            {{ key_ffi_converter }}.read(buf): {{ value_ffi_converter }}.read(buf)
            for i in range(count)
        }
