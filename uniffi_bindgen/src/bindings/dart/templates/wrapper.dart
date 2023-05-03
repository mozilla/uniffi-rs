// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!
{%- import "macros.dart" as dart %}

library api;

import "dart:convert";
import "dart:ffi";
import "dart:io" show Platform, File, Directory;
import "dart:typed_data";
import "package:ffi/ffi.dart";

{%- if ci.has_async_fns() %}
import "dart:async";
import "dart:isolate";
{%- endif %}

{%- for imported_class in self.imports() %}
import {{ imported_class }};
{%- endfor %}

// // Depending on the consumer's build setup, the low-level FFI code
// // might be in a separate module, or it might be compiled inline into
// // this module. This is a bit of light hackery to work with both.
// #if canImport({{ config.ffi_module_name() }})
// import {{ config.ffi_module_name() }}
// #endif

{% include "RustBufferTemplate.dart" %}
{% include "Helpers.dart" %}

// Public interface members begin here.
{{ type_helper_code }}
{# comment


/**
 * Top level initializers and tear down methods.
 *
 * This is generated by uniffi.
 */
public enum {{ config.module_name().borrow()|class_name }}Lifecycle {
    /**
     * Initialize the FFI and Rust library. This should be only called once per application.
     */
    func initialize() {
        {%- for initialization_fn in self.initialization_fns() %}
        {{ initialization_fn }}()
        {%- endfor %}
    }
}

endcomment #}


/// Main entry point to library.
class Api {
    /// Holds the symbol lookup function.
    final Pointer<T> Function<T extends NativeType>(String symbolName)
        _lookup;

    /// The symbols are looked up in [dynamicLibrary].
    Api(DynamicLibrary dynamicLibrary) : _lookup = dynamicLibrary.lookup;

    /// The symbols are looked up with [lookup].
    Api.fromLookup(
        Pointer<T> Function<T extends NativeType>(String symbolName)
            lookup)
        : _lookup = lookup;

    /// The library is loaded from the executable.
    factory Api.loadStatic() {
        return Api(DynamicLibrary.executable());
    }

    /// The library is dynamically loaded.
    factory Api.loadDynamic(String name) {
        return Api(DynamicLibrary.open(name));
    }

    /// The library is loaded based on platform conventions.
    factory Api.load() {
        String? name; {# FIXME: dynamic naming #}
        if (Platform.isLinux) name = "lib{{ config.cdylib_name() }}.so";
        if (Platform.isAndroid) name = "lib{{ config.cdylib_name() }}.so";
        if (Platform.isMacOS) name = "lib{{ config.cdylib_name() }}.dylib";
        if (Platform.isIOS) name = "";
        if (Platform.isWindows) name = "{{ config.cdylib_name() }}.dll";
        if (name == null) {
            throw UnsupportedError("\"This platform is not supported.\"");
        }
        if (name == "") {
            return Api.loadStatic();
        } else {
            return Api.loadDynamic(name);
        }
    }

    {%- for func in ci.function_definitions() %}
    {%- include "TopLevelFunctionTemplate.dart" %}
    {%- endfor %}
}