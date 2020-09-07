class RustError(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int32),
        ("message", ctypes.c_void_p),
    ]

    def free(self):
        rust_call_with_error(InternalError, _UniFFILib.{{ ci.ffi_string_free().name() }}, self.message)

    def __str__(self):
        return "RustError(code={}, message={})".format(
            self.code,
            str(ctypes.cast(self.message, ctypes.c_char_p).value, "utf-8"),
        )

class InternalError(Exception):
    @staticmethod
    def raise_err(code, message):
        raise InternalError(message)

{% for e in ci.iter_error_definitions() %}
class {{ e.name()|class_name_py }}:
    {%- for value in e.values() %}
    class {{ value|class_name_py }}(Exception):
        pass
    {%- endfor %}

    @staticmethod
    def raise_err(code, message):
        {%- for value in e.values() %}
        if code == {{ loop.index }}:
            raise {{ e.name()|class_name_py }}.{{ value|class_name_py }}(message)
        {% endfor %}
        raise Exception("Unknown error code")
{% endfor %}

def rust_call_with_error(error_class, fn, *args):
    error = RustError()
    error.code = 0

    args_with_error = args + (ctypes.byref(error),)
    result = fn(*args_with_error)
    if error.code != 0:
        message = str(error)
        error.free()

        error_class.raise_err(error.code, message)
    
    return result
