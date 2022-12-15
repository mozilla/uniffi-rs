{%- if func.is_async() %}
{%- match func.throws_type() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|type_name }}::class)
{%- else -%}
{%- endmatch %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

suspend fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}) = suspendCoroutine<{{ return_type|type_name }}> { continuation ->
    val rustFuture = {% call kt::to_ffi_call(func) %}
    
    class Waker: RustFutureWaker {
        override fun callback(env: Pointer?) {
            val polledResult =  {% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi_lowered }}{% when None %}Int{% endmatch %}ByReference()
            val isReady = rustCall() { _status ->
                _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_poll(
                    rustFuture,
                    this,
                    null, // env
                    polledResult,
                    _status
                )
            }
            
            if (isReady) {
                continuation.resume({{ return_type|lift_fn}}(polledResult.getValue()))
                rustCall() { _status ->
                    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_drop(rustFuture, _status)
                }
            }
        }
    }
    
    val waker = Waker()
    waker.callback(null)
    
    println("ennnd?")
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
