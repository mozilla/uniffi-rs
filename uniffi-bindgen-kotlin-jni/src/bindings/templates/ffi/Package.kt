@file:OptIn(kotlin.ExperimentalUnsignedTypes::class)
@file:Suppress("NOTHING_TO_INLINE")
package {{ package.name }}.uniffi

{%- for import in package.imports %}
import {{ import }}
{%- endfor %}

{%- for type_def in package.type_definitions %}
{%- if let TypeDefinition::External(ext) = type_def %}
{%- if !ext.self_type.lowers_to_primitive() %}
import {{ ext.package_name }}.{{ ext.self_type.lowered_type_kt() }}
{%- endif %}
{%- endif %}
{%- endfor %}

{%- include "Async.kt" %}
{%- let type_definitions = package.type_definitions %}
{%- include "TypeDefinitions.kt" %}

{# re-export specialized lift/lower functions #}
inline fun lowerBytesRef(v: java.nio.ByteBuffer): java.nio.ByteBuffer? {
    return uniffi.lowerBytesRef(v)
}

object Scaffolding {
    init {
        // Reference uniffi.Scaffolding to ensure it's init() function runs
        @Suppress("UNUSED_EXPRESSION")
        uniffi.Scaffolding
    }

    {%- for (jni_method_name, _, callable) in package.jni_methods() %}
    @JvmName("{{ jni_method_name }}")
    @JvmStatic external fun {{ jni_method_name }}(
        {%- for ffi_arg in callable.ffi_arguments_including_receiver() %}
        {{ ffi_arg.name_kt() }}: {{ ffi_arg.ty.type_kt() }},
        {%- endfor %}
    )
    {%- if callable.is_async %} : kotlin.Long
    {%- else %}
    {%- match callable.return_ffi() %}
    {%- when ReturnFfi::Primitive { ffi_type, .. } %} : {{ ffi_type.type_kt() }}
    {%- when ReturnFfi::Deconstruct { type_node, .. } %} : {{ type_node.type_kt }}
    {%- when ReturnFfi::Void %}
    {%- endmatch %}
    {%- endif %}
    {%- endfor %}

    {%- for cls in package.classes() %}
    @JvmStatic external fun {{ cls.jni_clone_name() }}(handle: kotlin.Long): kotlin.Long
    @JvmStatic external fun {{ cls.jni_free_name() }}(handle: kotlin.Long)
    {%- endfor  %}

    {%- for box_ in package.boxes() %}
    @JvmStatic external fun {{ box_.jni_from_ffi_values_name() }}(
        {%- for ffi_type in box_.inner.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    ): kotlin.Long
    @JvmStatic external fun {{ box_.jni_into_inner_name() }}(handle: kotlin.Long): {{ box_.inner.type_kt }}
    {%- endfor  %}

    {%- for result in package.kotlin_sync_callable_results() %}
    {%- if let Some(return_type) = result.return_type %}
    {%- if !return_type.lowers_to_primitive() %}
    @JvmStatic external fun {{ result.set_callback_return_fn() }}(
        resultPtr: kotlin.Long,
        {%- for ffi_type in return_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    )
    {%- endif %}
    {%- endif %}

    {%- if let Some(throws_type) = result.throws_type %}
    @JvmStatic external fun {{ result.set_callback_err_fn() }}(
        resultPtr: kotlin.Long,
        {%- for ffi_type in throws_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    )
    {%- endif %}
    {%- endfor %}

    {%- for rust_result in package.rust_async_callable_results() %}
    @JvmStatic external fun {{ rust_result.async_poll_fn() }}(
        rustFuture: kotlin.Long,
        continuation: kotlin.coroutines.Continuation<kotlin.Boolean>,
        {%- if rust_result.return_type.is_some() %}
        completion: {{ rust_result.async_complete_class() }},
        {%- endif %}
    ): Int
    @JvmStatic external fun {{ rust_result.async_cancel_fn() }}(rustFuture: kotlin.Long)
    @JvmStatic external fun {{ rust_result.async_free_fn() }}(rustFuture: kotlin.Long)
    {%- endfor %}

    {%- for result in package.kotlin_async_callable_results() %}
    @JvmStatic external fun {{ result.async_complete_success_fn() }}(
        oneshotHandle: kotlin.Long,
        {%- if let Some(return_type) = result.return_type %}
        {%- for ffi_type in return_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
        {%- endif %}
    )
    {%- if let Some(throws_type) = result.throws_type %}
    @JvmStatic external fun {{ result.async_complete_error_fn() }}(
        oneshotHandle: kotlin.Long,
        {%- for ffi_type in throws_type.ffi_types %}
        v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
        {%- endfor %}
    )
    {%- endif %}
    @JvmStatic external fun {{ result.async_complete_unexpected_error_fn() }}(
        oneshotHandle: kotlin.Long,
    )
    {%- endfor %}
}
