{%- let obj = ci|get_object_definition(name) %}
{%- if self.include_once_check("ObjectRuntime.kt") %}{% include "ObjectRuntime.kt" %}{% endif %}
{%- let (interface_name, impl_class_name) = obj|object_names %}
{%- let methods = obj.methods() %}

{% include "Interface.kt" %}

class {{ impl_class_name }}(
    pointer: Pointer
) : FFIObject(pointer), {{ interface_name }}{

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    constructor({% call kt::arg_list_decl(cons) -%}) :
        this({% call kt::to_ffi_call(cons) %})
    {%- when None %}
    {%- endmatch %}

    /**
     * Disconnect the object from the underlying Rust object.
     *
     * It can be called more than once, but once called, interacting with the object
     * causes an `IllegalStateException`.
     *
     * Clients **must** call this method once done with the object, or cause a memory leak.
     */
    override protected fun freeRustArcPtr() {
        rustCall() { status ->
            _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(this.pointer, status)
        }
    }

    {% for meth in obj.methods() -%}
    {%- match meth.throws_type() -%}
    {%- when Some with (throwable) %}
    @Throws({{ throwable|error_type_name }}::class)
    {%- else -%}
    {%- endmatch -%}
    {%- if meth.is_async() %}
    @Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE")
    override suspend fun {{ meth.name()|fn_name }}({%- call kt::arg_list_decl(meth) -%}){% match meth.return_type() %}{% when Some with (return_type) %} : {{ return_type|type_or_iface_name }}{% when None %}{%- endmatch %} {
        return uniffiRustCallAsync(
            callWithPointer { thisPtr ->
                _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}(
                    thisPtr,
                    {% call kt::arg_list_lowered(meth) %}
                )
            },
            { future, continuation -> _UniFFILib.INSTANCE.{{ meth.ffi_rust_future_poll(ci) }}(future, continuation) },
            { future, status -> _UniFFILib.INSTANCE.{{ meth.ffi_rust_future_complete(ci) }}(future, status) },
            { future -> _UniFFILib.INSTANCE.{{ meth.ffi_rust_future_free(ci) }}(future) },
            // lift function
            {%- match meth.return_type() %}
            {%- when Some(return_type) %}
            { {{ return_type|lift_fn }}(it) },
            {%- when None %}
            { Unit },
            {% endmatch %}
            // Error FFI converter
            {%- match meth.throws_type() %}
            {%- when Some(e) %}
            {{ e|error_type_name }}.ErrorHandler,
            {%- when None %}
            NullCallStatusErrorHandler,
            {%- endmatch %}
        )
    }
    {%- else -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) -%}
    override fun {{ meth.name()|fn_name }}({% call kt::arg_list_protocol(meth) %}): {{ return_type|type_or_iface_name }} =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ return_type|lift_fn }}(it)
        }

    {%- when None -%}
    override fun {{ meth.name()|fn_name }}({% call kt::arg_list_protocol(meth) %}) =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }
    {% endmatch %}
    {% endif %}
    {% endfor %}

    {% if !obj.alternate_constructors().is_empty() -%}
    companion object {
        {% for cons in obj.alternate_constructors() -%}
        fun {{ cons.name()|fn_name }}({% call kt::arg_list_decl(cons) %}): {{ impl_class_name }} =
            {{ impl_class_name }}({% call kt::to_ffi_call(cons) %})
        {% endfor %}
    }
    {% else %}
    companion object
    {% endif %}
}

{%- if obj.is_trait_interface() %}
{%- let callback_handler_class = format!("UniffiCallbackInterface{}", name) %}
{%- let callback_handler_obj = format!("uniffiCallbackInterface{}", name) %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.kt" %}
{%- endif %}

public object {{ obj|ffi_converter_name }}: FfiConverter<{{ type_name }}, Pointer> {
    {%- if obj.is_trait_interface() %}
    internal val handleMap = ConcurrentHandleMap<{{ interface_name }}>()
    {%- endif %}

    override fun lower(value: {{ type_name }}): Pointer {
        {%- match obj.imp() %}
        {%- when ObjectImpl::Struct %}
        return value.callWithPointer { it }
        {%- when ObjectImpl::Trait %}
        return Pointer(handleMap.insert(value))
        {%- endmatch %}
    }

    override fun lift(value: Pointer): {{ type_name }} {
        return {{ impl_class_name }}(value)
    }

    override fun read(buf: ByteBuffer): {{ type_name }} {
        // The Rust code always writes pointers as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(Pointer(buf.getLong()))
    }

    override fun allocationSize(value: {{ type_name }}) = 8

    override fun write(value: {{ type_name }}, buf: ByteBuffer) {
        // The Rust code always expects pointers written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(Pointer.nativeValue(lower(value)))
    }
}
