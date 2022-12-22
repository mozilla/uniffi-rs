{%- let obj = ci.get_object_definition(name).unwrap() %}
{%- if self.include_once_check("ObjectRuntime.kt") %}{% include "ObjectRuntime.kt" %}{% endif %}
{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.atomic.AtomicBoolean") }}

public interface {{ type_name }}Interface {
    {% for meth in obj.methods() -%}
    {%- if meth.is_async() -%}
    suspend fun {{ meth.name()|fn_name }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_name -}}
    {%- when None -%}
    {%- endmatch -%}
    {%- else -%}
    {%- match meth.throws_type() -%}
    {%- when Some with (throwable) %}
    @Throws({{ throwable|type_name }}::class)
    {%- else -%}
    {%- endmatch %}
    fun {{ meth.name()|fn_name }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch -%}
    {%- endif -%}
    {% endfor %}
}

class {{ type_name }}(
    pointer: Pointer
) : FFIObject(pointer), {{ type_name }}Interface {

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
    {%- if meth.is_async() -%}
    override suspend fun {{ meth.name()|fn_name }}({%- call kt::arg_list_protocol(meth) -%}){% match meth.return_type() %}{% when Some with (return_type) %}: {{ return_type|type_name }}{% when None %}{% endmatch %} {
        class Waker: RustFutureWaker {
            override fun callback(envCStructure: RustFutureWakerEnvironmentCStructure?) {
                val hash = envCStructure!!.hash
                val env = _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.get(hash)!!

                env.coroutineScope.launch {
                    val polledResult =  {% match meth.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi_lowered }}{% when None %}Pointer{% endmatch %}ByReference()
                    val isReady = rustCall() { _status ->
                        _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}_poll(
                            env.rustFuture,
                            env.waker,
                            env.selfAsCStructure,
                            polledResult,
                            _status
                        )
                    }

                    if (isReady) {
                        @Suppress("UNCHECKED_CAST")
                        {% match meth.return_type() -%}
                        {%- when Some with (return_type) -%}
                            (env.continuation as Continuation<{{ return_type|type_name }}>).resume({{ return_type|lift_fn}}(polledResult.getValue()))
                        {%- when None -%}
                            (env.continuation as Continuation<Unit>).resume(Unit)
                        {%- endmatch %}

                        _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.remove(hash)
                        rustCall() { _status ->
                            _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}_drop(env.rustFuture, _status)
                        }
                    }
                }
            }
        }

        val result: {% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}Unit{% endmatch %}

        coroutineScope {
            result = suspendCoroutine<{% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}Unit{% endmatch %}> { continuation ->
                val rustFuture = callWithPointer {
                    {% call kt::to_ffi_call_with_prefix("it", meth) %}
                }

                val env = RustFutureWakerEnvironment(rustFuture, continuation, Waker(), RustFutureWakerEnvironmentCStructure(), this)
                val envHash = env.hashCode()
                env.selfAsCStructure.hash = envHash

                _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.put(envHash, env)

                val waker = Waker()
                waker.callback(env.selfAsCStructure)
            }
        }

        return result
    }

    {%- else -%}

    {%- match meth.throws_type() -%}
    {%- when Some with (throwable) %}
    @Throws({{ throwable|type_name }}::class)
    {%- else -%}
    {%- endmatch %}

    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    override fun {{ meth.name()|fn_name }}({% call kt::arg_list_protocol(meth) %}): {{ return_type|type_name }} =
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
        fun {{ cons.name()|fn_name }}({% call kt::arg_list_decl(cons) %}): {{ type_name }} =
            {{ type_name }}({% call kt::to_ffi_call(cons) %})
        {% endfor %}
    }
    {% endif %}
}

public object {{ obj|ffi_converter_name }}: FfiConverter<{{ type_name }}, Pointer> {
    override fun lower(value: {{ type_name }}): Pointer = value.callWithPointer { it }

    override fun lift(value: Pointer): {{ type_name }} {
        return {{ type_name }}(value)
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
