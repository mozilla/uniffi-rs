# Define some ctypes FFI types that we use in the library

"""
ctypes type for the foreign executor callback.  This is a built-in interface for scheduling
tasks

Args:
  executor: opaque c_size_t value representing the eventloop
  delay: delay in ms
  task: function pointer to the task callback
  task_data: void pointer to the task callback data

Normally we should call task(task_data) after the detail.
However, when task is NULL this indicates that Rust has dropped the ForeignExecutor and we should
decrease the EventLoop refcount.
"""
UNIFFI_FOREIGN_EXECUTOR_CALLBACK_T = ctypes.CFUNCTYPE(None, ctypes.c_size_t, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_void_p)

"""
Function pointer for a Rust task, which a callback function that takes a opaque pointer
"""
UNIFFI_RUST_TASK = ctypes.CFUNCTYPE(None, ctypes.c_void_p)

from pathlib import Path

def loadIndirect():
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

    libname = libname.format("{{ config.cdylib_name() }}")
    path = str(Path(__file__).parent / libname)
    lib = ctypes.cdll.LoadLibrary(path)
    uniffi_check_contract_api_version(lib)
    uniffi_check_api_checksums(lib)
    return lib

def uniffi_check_contract_api_version(lib):
    # Get the bindings contract version from our ComponentInterface
    bindings_contract_version = {{ ci.uniffi_contract_version() }}
    # Get the scaffolding contract version by calling the into the dylib
    scaffolding_contract_version = lib.{{ ci.ffi_uniffi_contract_version().name() }}()
    if bindings_contract_version != scaffolding_contract_version:
        raise InternalError("UniFFI contract version mismatch: try cleaning and rebuilding your project")

def uniffi_check_api_checksums(lib):
    {%- for (name, expected_checksum) in ci.iter_checksums() %}
    if lib.{{ name }}() != {{ expected_checksum }}:
        raise InternalError("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    {%- else %}
    pass
    {%- endfor %}

# A ctypes library to expose the extern-C FFI definitions.
# This is an implementation detail which will be called internally by the public API.

_UniFFILib = loadIndirect()
{%- for func in ci.iter_ffi_function_definitions() %}
{%- if func.is_async() %}

_UniFFILib.{{ func.name() }}.argtypes = (
    {%- call py::arg_list_ffi_decl(func) -%}
)
_UniFFILib.{{ func.name() }}.restype = ctypes.POINTER(RustFuture)

_UniFFILib.{{ func.name() }}_poll.argtypes = (
    ctypes.POINTER(RustFuture),
    FUTURE_WAKER_T,
    FUTURE_WAKER_ENVIRONMENT_T,
    {% match func.return_type() %}{% when Some with (type_) %}ctypes.POINTER({{ type_|ffi_type_name }}){% when None %}ctypes.c_void_p{% endmatch %},
    {%- if func.has_rust_call_status_arg() %}ctypes.POINTER(RustCallStatus), {% endif %}
)
_UniFFILib.{{ func.name() }}_poll.restype = ctypes.c_bool

_UniFFILib.{{ func.name() }}_drop.argtypes = (
    ctypes.POINTER(RustFuture),
    ctypes.POINTER(RustCallStatus),
)
_UniFFILib.{{ func.name() }}_drop.restype = None

{%- else %}

_UniFFILib.{{ func.name() }}.argtypes = (
    {%- call py::arg_list_ffi_decl(func) -%}
)
_UniFFILib.{{ func.name() }}.restype = {% match func.return_type() %}{% when Some with (type_) %}{{ type_|ffi_type_name }}{% when None %}None{% endmatch %}

{%- endif %}
{%- endfor %}
