{%- let obj = ci|get_object_definition(name) %}
{%- if self.include_once_check("ObjectRuntime.kt") %}{% include "ObjectRuntime.kt" %}{% endif %}
{%- let (interface_name, impl_class_name) = obj|object_names(ci) %}
{%- let methods = obj.methods() %}

{% include "Interface.kt" %}

open class {{ impl_class_name }} : FFIObject, {{ interface_name }} {

    constructor(pointer: Pointer): super(pointer)

    /**
     * This constructor can be used to instantiate a fake object.
     *
     * **WARNING: Any object instantiated with this constructor cannot be passed to an actual Rust-backed object.**
     * Since there isn't a backing [Pointer] the FFI lower functions will crash.
     * @param noPointer Placeholder value so we can have a constructor separate from the default empty one that may be
     *   implemented for classes extending [FFIObject].
     */
    constructor(noPointer: NoPointer): super(noPointer)

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
        this.pointer?.let { ptr ->
            rustCall() { status ->
                _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(ptr, status)
            }
        }
    }

    {% for meth in obj.methods() -%}
    {%- match meth.throws_type() -%}
    {%- when Some with (throwable) %}
    @Throws({{ throwable|type_name(ci) }}::class)
    {%- else -%}
    {%- endmatch -%}
    {%- if meth.is_async() %}
    @Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE")
    override suspend fun {{ meth.name()|fn_name }}(
        {%- call kt::arg_list_decl(meth) -%}
    ){% match meth.return_type() %}{% when Some with (return_type) %} : {{ return_type|type_name(ci) }}{% when None %}{%- endmatch %} {
        return uniffiRustCallAsync(
            callWithPointer { thisPtr ->
                _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}(
                    thisPtr,
                    {% call kt::arg_list_lowered(meth) %}
                )
            },
            {{ meth|async_poll(ci) }},
            {{ meth|async_complete(ci) }},
            {{ meth|async_free(ci) }},
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
            {{ e|type_name(ci) }}.ErrorHandler,
            {%- when None %}
            NullCallStatusErrorHandler,
            {%- endmatch %}
        )
    }
    {%- else -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) -%}
    override fun {{ meth.name()|fn_name }}(
        {%- call kt::arg_list_protocol(meth) -%}
    ): {{ return_type|type_name(ci) }} =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ return_type|lift_fn }}(it)
        }

    {%- when None -%}
    override fun {{ meth.name()|fn_name }}(
        {%- call kt::arg_list_protocol(meth) -%}
    ) =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }
    {% endmatch %}
    {% endif %}
    {% endfor %}

    {%- for tm in obj.uniffi_traits() %}
    {%-     match tm %}
    {%-         when UniffiTrait::Display { fmt } %}
    override fun toString(): String =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", fmt) %}
        }.let {
            {{ fmt.return_type().unwrap()|lift_fn }}(it)
        }
    {%-         when UniffiTrait::Eq { eq, ne } %}
    {# only equals used #}
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is {{ impl_class_name}}) return false
        return callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", eq) %}
        }.let {
            {{ eq.return_type().unwrap()|lift_fn }}(it)
        }
    }
    {%-         when UniffiTrait::Hash { hash } %}
    override fun hashCode(): Int =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", hash) %}
        }.let {
            {{ hash.return_type().unwrap()|lift_fn }}(it).toInt()
        }
    {%-         else %}
    {%-     endmatch %}
    {%- endfor %}

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
