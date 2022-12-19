{%- if func.is_async() %}
{%- match func.throws_type() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|type_name }}::class)
{%- else -%}
{%- endmatch %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

suspend fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    class Waker: RustFutureWaker {
        override fun callback(envCStructure: RustFutureWakerEnvironmentCStructure?) {
            val hash = envCStructure!!.hash
            val env = _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.get(hash)!!

            env.coroutineScope.launch {
                val polledResult =  {% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi_lowered }}{% when None %}Byte{% endmatch %}ByReference()
                val isReady = rustCall() { _status ->
                    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_poll(
                        env.rustFuture,
                        env.waker,
                        env.selfAsCStructure,
                        polledResult,
                        _status
                    )
                }

                if (isReady) {
                    @Suppress("UNCHECKED_CAST")
                    (env.continuation as Continuation<{{ return_type|type_name }}>).resume({{ return_type|lift_fn}}(polledResult.getValue()))

                    _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.remove(hash)
                    rustCall() { _status ->
                        _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_drop(env.rustFuture, _status)
                    }
                }
            }
        }
    }

    val result: {{ return_type|type_name }}

    coroutineScope {
        result = suspendCoroutine<{{ return_type|type_name }}> { continuation ->
            val rustFuture = {% call kt::to_ffi_call(func) %}

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

{% when None %}

// TODO
{% endmatch %}
{%- else %}
{%- match func.throws_type() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|type_name }}::class)
{%- else -%}
{%- endmatch %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    return {{ return_type|lift_fn }}({% call kt::to_ffi_call(func) %})
}

{% when None %}

fun {{ func.name()|fn_name }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
{%- endif %}
