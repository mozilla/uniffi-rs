// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// The following structs are used to implement the lowest level
// of the FFI, and thus useful to multiple uniffied crates.
// We ensure they are declared exactly once, with a header guard, UNIFFI_SHARED_H.
#ifdef UNIFFI_SHARED_H
    // We also try to prevent mixing versions of shared uniffi header structs.
    // If you add anything to the #else block, you must increment the version suffix in UNIFFI_SHARED_HEADER_V4
    #ifndef UNIFFI_SHARED_HEADER_V4
        #error Combining helper code from multiple versions of uniffi is not supported
    #endif // ndef UNIFFI_SHARED_HEADER_V4
#else
#define UNIFFI_SHARED_H
#define UNIFFI_SHARED_HEADER_V4
// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️

typedef struct RustBuffer
{
    int32_t capacity;
    int32_t len;
    uint8_t *_Nullable data;
} RustBuffer;

typedef int32_t (*ForeignCallback)(uint64_t, int32_t, const uint8_t *_Nonnull, int32_t, RustBuffer *_Nonnull);

typedef struct ForeignBytes
{
    int32_t len;
    const uint8_t *_Nullable data;
} ForeignBytes;

// Error definitions
typedef struct RustCallStatus {
    int8_t code;
    RustBuffer errorBuf;
} RustCallStatus;

// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️
#endif // def UNIFFI_SHARED_H

// Define FFI callback types
{%- for callback in ci.ffi_callback_definitions() %}
typedef
    {%- match callback.return_type() %}{% when Some(return_type) %} {{ return_type|ffi_type_name }} {% when None %} void {% endmatch -%}
    (*{{ callback.name()|ffi_callback_name }})(
        {%- for arg in callback.arguments() -%}
        {{ arg.type_().borrow()|header_ffi_type_name }}
        {%- if !loop.last %}, {% endif %}
        {%- endfor -%}
    );
{%- endfor %}

// Scaffolding functions
{%- for func in ci.iter_ffi_function_definitions() %}
{% match func.return_type() -%}{%- when Some with (type_) %}{{ type_|header_ffi_type_name }}{% when None %}void{% endmatch %} {{ func.name() }}(
    {%- if func.arguments().len() > 0 %}
        {%- for arg in func.arguments() %}
            {{- arg.type_().borrow()|header_ffi_type_name }} {{ arg.name() -}}{% if !loop.last || func.has_rust_call_status_arg() %}, {% endif %}
        {%- endfor %}
        {%- if func.has_rust_call_status_arg() %}RustCallStatus *_Nonnull out_status{% endif %}
    {%- else %}
        {%- if func.has_rust_call_status_arg() %}RustCallStatus *_Nonnull out_status{%- else %}void{% endif %}
    {% endif %}
);
{%- endfor %}

{% import "macros.swift" as swift %}
