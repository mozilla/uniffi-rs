// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

// swiftlint:disable all

{%- call swift::docstring_value(ci.namespace_docstring(), 0) %}

{%- import "macros.swift" as swift %}
import Foundation
{%- for imported_class in self.imports() %}
import {{ imported_class }}
{%- endfor %}

// Depending on the consumer's build setup, the low-level FFI code
// might be in a separate module, or it might be compiled inline into
// this module. This is a bit of light hackery to work with both.
#if canImport({{ config.ffi_module_name() }})
import {{ config.ffi_module_name() }}
#endif

{% include "RustBufferTemplate.swift" %}
{% include "Helpers.swift" %}
{% include "HandleMap.swift" %}

// Public interface members begin here.
{{ type_helper_code }}

{%- if ci.has_async_fns() %}
{% include "Async.swift" %}
{%- endif %}

{%- for func in ci.function_definitions() %}
{%- include "TopLevelFunctionTemplate.swift" %}
{%- endfor %}

private enum InitializationResult {
    case ok
    case contractVersionMismatch
    case apiChecksumMismatch
}
// Use a global variable to perform the versioning checks. Swift ensures that
// the code inside is only computed once.
private let initializationResult: InitializationResult = {
    // Get the bindings contract version from our ComponentInterface
    let bindings_contract_version = {{ ci.uniffi_contract_version() }}
    // Get the scaffolding contract version by calling the into the dylib
    let scaffolding_contract_version = {{ ci.ffi_uniffi_contract_version().name() }}()
    if bindings_contract_version != scaffolding_contract_version {
        return InitializationResult.contractVersionMismatch
    }

{%- if !config.omit_checksums %}
    {%- for (name, expected_checksum) in ci.iter_checksums() %}
    if ({{ name }}() != {{ expected_checksum }}) {
        return InitializationResult.apiChecksumMismatch
    }
    {%- endfor %}
{%- endif %}

    {% for fn in self.initialization_fns() -%}
    {{ fn }}()
    {% endfor -%}

    return InitializationResult.ok
}()

// Make the ensure init function public so that other modules which have external type references to
// our types can call it.
public func {{ ensure_init_fn_name }}() {
    switch initializationResult {
    case .ok:
        break
    case .contractVersionMismatch:
        fatalError("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    case .apiChecksumMismatch:
        fatalError("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
}

// swiftlint:enable all
