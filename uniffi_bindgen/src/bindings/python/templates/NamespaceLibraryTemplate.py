# This is how we find and load the dynamic library provided by the component.
# For now we just look it up by name.
#
# XXX TODO: This will probably grow some magic for resolving megazording in future.
# E.g. we might start by looking for the named component in `libuniffi.so` and if
# that fails, fall back to loading it separately from `lib${componentName}.so`.

def loadIndirect(componentName):
    if sys.platform == "linux":
        libname = "libuniffi_{}.so"
    elif sys.platform == "darwin":
        libname = "libuniffi_{}.dylib"
    elif sys.platform.startswith("win"):
        libname = "libuniffi_{}.dll"
    return getattr(ctypes.cdll, libname.format(componentName))

# A ctypes library to expose the extern-C FFI definitions.
# This is an implementation detail which will be called internally by the public API.

_UniFFILib = loadIndirect(componentName="{{ ci.namespace() }}")
{%- for func in ci.iter_ffi_function_definitions() %}
_UniFFILib.{{ func.name() }}.argtypes = (
    {%- call py::arg_list_ffi_decl(func) -%}
)
_UniFFILib.{{ func.name() }}.restype = {% match func.return_type() %}{% when Some with (type_) %}{{ type_|type_ffi }}{% when None %}None{% endmatch %}
{%- endfor %}