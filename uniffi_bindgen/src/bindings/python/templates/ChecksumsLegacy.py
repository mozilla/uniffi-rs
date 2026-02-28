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


_uniffi_check_contract_api_version(_UniffiLib)
_uniffi_check_api_checksums(_UniffiLib)
