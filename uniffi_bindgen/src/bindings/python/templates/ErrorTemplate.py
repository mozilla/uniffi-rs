class RustError(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int32),
        ("message", ctypes.c_char_p),
    ]

    def __str__(self):
        return "RustError(code={}, message={})".format(self.code, str(self.message, "utf-8")) 

{% for e in ci.iter_error_definitions() %}
class {{e.name()}}:
    {%- for value in e.values() %}
    class {{value}}(Exception):
        pass
    {%- endfor %}

    @staticmethod
    def raise_err(err):
        {%- for value in e.values() %}
        if err.code == {{loop.index}}:
            raise {{e.name()}}.{{value}}(err.__str__())
        {% endfor %}
        raise Exception("Unknown {{e.name()}} error code")
{% endfor %}

class RustErrorHelper:
    def __init__(self):
        self.err = RustError()

    def __apply__(self, fn):
        return fn(ctypes.byref(self.err))

    def __raise__(self):
    {%- for e in ci.iter_error_definitions() %}
        if self.err.code == {{loop.index}}:
            return {{e.name()}}.raise_err(self.err)
    {% endfor %}
        return

    def __reset__(self):
        del self.err
        self.err = RustError()

    def try_raise(self, fn):
        self.__reset__()
        result = self.__apply__(fn)
        self.__raise__()
        
        return result

_RustErrorHelper = RustErrorHelper()
