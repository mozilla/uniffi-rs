# Define some ctypes FFI types that we use in the library

"""
Function pointer for a Rust task, which a callback function that takes a opaque pointer
"""
_UNIFFI_RUST_TASK = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_int8)

def _uniffi_future_callback_t(return_type):
    """
    Factory function to create callback function types for async functions
    """
    return ctypes.CFUNCTYPE(None, ctypes.c_uint64, return_type, _UniffiRustCallStatus)

def _uniffi_load_indirect():
    """
    This is how we find and load the dynamic library provided by the component.
    For now we just look it up by name.
    """
    if sys.platform == "darwin":
        libname = "lib{}.dylib"
    elif sys.platform.startswith("win"):
        # As of python3.8, ctypes does not seem to search $PATH when loading DLLs.
        # We could use `os.add_dll_directory` to configure the search path, but
        # it doesn't feel right to mess with application-wide settings. Let's
        # assume that the `.dll` is next to the `.py` file and load by full path.
        libname = os.path.join(
            os.path.dirname(__file__),
            "{}.dll",
        )
    else:
        # Anything else must be an ELF platform - Linux, *BSD, Solaris/illumos
        libname = "lib{}.so"

    libname = libname.format("{{ cdylib_name }}")
    path = os.path.join(os.path.dirname(__file__), libname)
    lib = ctypes.cdll.LoadLibrary(path)
    return lib

def _uniffi_check_contract_api_version(lib):
    # Get the bindings contract version from our ComponentInterface
    bindings_contract_version = {{ correct_contract_version }}
    # Get the scaffolding contract version by calling the into the dylib
    scaffolding_contract_version = lib.{{ ffi_uniffi_contract_version.0 }}()
    if bindings_contract_version != scaffolding_contract_version:
        raise InternalError("UniFFI contract version mismatch: try cleaning and rebuilding your project")

def _uniffi_check_api_checksums(lib):
    {%- for checksum in checksums %}
    if lib.{{ checksum.fn_name.0 }}() != {{ checksum.checksum }}:
        raise InternalError("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    {%- else %}
    pass
    {%- endfor %}

# A ctypes library to expose the extern-C FFI definitions.
# This is an implementation detail which will be called internally by the public API.

_UniffiLib = _uniffi_load_indirect()

{%- for def in ffi_definitions %}
{%- match def %}
{%- when FfiDefinition::FunctionType(function_type) %}
{{ function_type.name.0 }} = ctypes.CFUNCTYPE(
    {%- match function_type.return_type.ty %}
    {%- when Some(return_type) %}{{ return_type.type_name }},
    {%- when None %}None,
    {%- endmatch %}
    {%- for arg in function_type.arguments -%}
    {{ arg.ty.type_name }},
    {%- endfor -%}
    {%- if function_type.has_rust_call_status_arg %}
    ctypes.POINTER(_UniffiRustCallStatus),
    {%- endif %}
)
{%- when FfiDefinition::Struct(ffi_struct) %}
class {{ ffi_struct.name.0 }}(ctypes.Structure):
    _fields_ = [
        {%- for field in ffi_struct.fields %}
        ("{{ field.name }}", {{ field.ty.type_name }}),
        {%- endfor %}
    ]
{%- when FfiDefinition::RustFunction(func) %}
_UniffiLib.{{ func.name.0 }}.argtypes = (
    {%- for arg in func.arguments %}
    {{ arg.ty.type_name }},
    {%- endfor %}
    {%- if func.has_rust_call_status_arg %}
    ctypes.POINTER(_UniffiRustCallStatus),
    {%- endif %}
)
_UniffiLib.{{ func.name.0 }}.restype = {% match func.return_type.ty %}{% when Some(ffi_type) %}{{ ffi_type.type_name }}{% when None %}None{% endmatch %}
{%- endmatch %}
{%- endfor %}

{# Ensure to call the contract verification only after we defined all functions. -#}
_uniffi_check_contract_api_version(_UniffiLib)
# _uniffi_check_api_checksums(_UniffiLib)
