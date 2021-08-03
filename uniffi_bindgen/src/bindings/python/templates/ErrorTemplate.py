class InternalError(Exception):
    pass

class RustCallStatus(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int8),
        ("error_buf", RustBuffer),
    ]

    # These match the values from the uniffi::rustcalls module
    CALL_SUCCESS = 0
    CALL_ERROR = 1
    CALL_PANIC = 2

    def __str__(self):
        if self.code == RustCallStatus.CALL_SUCCESS:
            return "RustCallStatus(CALL_SUCCESS)"
        elif self.code == RustCallStatus.CALL_ERROR:
            return "RustCallStatus(CALL_ERROR)"
        elif self.code == RustCallStatus.CALL_PANIC:
            return "RustCallStatus(CALL_SUCCESS)"
        else:
            return "RustCallStatus(<invalid code>)"
{%- for e in ci.iter_error_definitions() %}

class {{ e.name()|class_name_py }}:
    # Each variant is a nested class of the error itself.
    {%- for variant in e.variants() %}

    class {{ variant.name()|class_name_py }}(Exception):
        def __init__(self{% for field in variant.fields() %}, {{ field.name()|var_name_py }}{% endfor %}):
            {%- if variant.has_fields() %}
            {%- for field in variant.fields() %}
            self.{{ field.name()|var_name_py }} = {{ field.name()|var_name_py }}
            {%- endfor %}
            {%- else %}
            pass
            {%- endif %}

        def __str__(self):
            {%- if variant.has_fields() %}
            field_parts = [
                {%- for field in variant.fields() %}
                '{{ field.name() }}={!r}'.format(self.{{ field.name() }}),
                {%- endfor %}
            ]
            return "{{ e.name()|class_name_py }}.{{ variant.name()|class_name_py }}({})".format(', '.join(field_parts))
            {%- else %}
            return "{{ e.name()|class_name_py }}.{{ variant.name()|class_name_py }}"
            {%- endif %}
    {%- endfor %}
{%- endfor %}

# Map error classes to the RustBufferTypeBuilder method to read them
_error_class_to_reader_method = {
{%- for e in ci.iter_error_definitions() %}
{%- let typ=ci.get_type(e.name()).unwrap() %}
{%- let canonical_type_name = typ.canonical_name()|class_name_py %}
    {{ e.name()|class_name_py }}: RustBufferTypeReader.read{{ canonical_type_name }},
{%- endfor %}
}

def consume_buffer_into_error(error_class, rust_buffer):
    reader_method = _error_class_to_reader_method[error_class]
    with rust_buffer.consumeWithStream() as stream:
        return reader_method(stream)

def rust_call(fn, *args):
    # Call a rust function
    return rust_call_with_error(None, fn, *args)

def rust_call_with_error(error_class, fn, *args):
    # Call a rust function and handle any errors
    #
    # This function is used for rust calls that return Result<> and therefore can set the CALL_ERROR status code.
    # error_class must be set to the error class that corresponds to the result.
    call_status = RustCallStatus(code=0, error_buf=RustBuffer(0, 0, None))

    args_with_error = args + (ctypes.byref(call_status),)
    result = fn(*args_with_error)
    if call_status.code == RustCallStatus.CALL_SUCCESS:
        return result
    elif call_status.code == RustCallStatus.CALL_ERROR:
        if error_class is None:
            call_status.err_buf.contents.free()
            raise InternalError("rust_call_with_error: CALL_ERROR, but no error class set")
        else:
            raise consume_buffer_into_error(error_class, call_status.error_buf)
    elif call_status.code == RustCallStatus.CALL_PANIC:
        # When the rust code sees a panic, it tries to construct a RustBuffer
        # with the message.  But if that code panics, then it just sends back
        # an empty buffer.
        if call_status.error_buf.len > 0:
            msg = call_status.error_buf.consumeIntoString()
        else:
            msg = "Unknown rust panic"
        raise InternalError(msg)
    else:
        raise InternalError("Invalid RustCallStatus code: {}".format(
            call_status.code))
