{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call`
#}

{%- macro to_ffi_call(func) -%}
    {%- match func.throws_type() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|type_name}})
    {%- else %}
    rustCall()
    {%- endmatch %} { _status ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %} _status)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) %}
    {%- match func.throws_type() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|type_name}})
    {%- else %}
    rustCall()
    {%- endmatch %} { _status ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) %}{% if func.arguments().len() > 0 %}, {% endif %} _status)
}
{%- endmacro %}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg|lower_fn }}({{ arg.name()|var_name }})
        {%- if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in kotlin declarations of methods, functions and constructors.
// Note the var_name and type_name filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|render_literal(arg) }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{% macro arg_list_protocol(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}
{#-
// Arglist as used in the _UniFFILib function declarations.
// Note unfiltered name but ffi_type_name filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|var_name }}: {{ arg.type_().borrow()|ffi_type_name -}},
    {%- endfor %}
    {%- if func.has_rust_call_status_arg() %}_uniffi_out_err: RustCallStatus, {% endif %}
{%- endmacro -%}

// Macro for destroying fields
{%- macro destroy_fields(member) %}
    Disposable.destroy(
    {%- for field in member.fields() %}
        this.{{ field.name()|var_name }}{%- if !loop.last %}, {% endif -%}
    {% endfor -%})
{%- endmacro -%}

{%- macro ffi_function_definition(func) %}
fun {{ func.name()|fn_name }}(
    {%- call arg_list_ffi_decl(func) %}
){%- match func.return_type() -%}{%- when Some with (type_) %}: {{ type_|ffi_type_name }}{% when None %}: Unit{% endmatch %}
{% endmacro %}
{%- macro async_func(func) -%}
    {%- call _async_func_or_method(func, false) -%}
{%- endmacro -%}

// Asny function and method.
{%- macro async_meth(meth) -%}
    {% call _async_func_or_method(meth, true) -%}
{%- endmacro -%}

{%- macro _async_func_or_method(func, is_meth) -%}
{% if is_meth %}override {% endif -%}
suspend fun {{ func.name()|fn_name }}(
    {%- if is_meth -%}
        {%- call arg_list_protocol(func) %}
    {%- else -%}
        {%- call arg_list_decl(func) %}
    {%- endif -%}
)
{%- match func.return_type() %}
{%- when Some with (return_type) -%}
    : {{ return_type|type_name }}
{%- when None %}
{%- endmatch %} {
    {# JNA defines callbacks as a class with a `callback` method -#}
    class Waker: RustFutureWaker {
        private val lock = Semaphore(1)

        override fun callback(envCStructure: RustFutureWakerEnvironmentCStructure?) {
            if (envCStructure == null) {
                return;
            }

            val hash = envCStructure.hash
            val env = _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.get(hash)

            if (env == null) {
                {# The future has been resolved already -#}
                return
            }

            {# Schedule a poll -#}
            env.coroutineScope.launch {
                {# Run one poll at a time -#}
                lock.withPermit {
                    if (!_UniFFILib.FUTURE_WAKER_ENVIRONMENTS.containsKey(hash)) {
                        {# The future has been resolved by a previous call -#}
                        return@withPermit
                    }

                    {# Cast the continuation to its appropriate type -#}
                    @Suppress("UNCHECKED_CAST")
                    val continuation = {% match func.return_type() -%}
                    {%- when Some with (return_type) -%}
                        env.continuation as Continuation<{{ return_type|type_name }}>
                    {%- when None -%}
                        env.continuation as Continuation<Unit>
                    {%- endmatch %}

                    {# Declare the `polledResult` variable: `_T_ByReference` where -#}
                    {#- `_T_` is the return type, or `PointerByReference` -#}
                    val polledResult = {% match func.ffi_func().return_type() -%}
                    {%- when Some with (return_type) -%}
                        {{ return_type|type_ffi_lowered }}
                    {%- when None -%}
                        Pointer
                    {%- endmatch %}ByReference()

                    {# Try to poll, catch exceptions if the future has thrown -#}
                    try {
                        {# Poll the future! -#}
                        val isReady = {% match func.throws_type() -%}
                        {%- when Some with (error) -%}
                            rustCallWithError({{ error|type_name }})
                        {%- when None -%}
                            rustCall()
                        {%- endmatch %}
                        { _status ->
                            _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_poll(
                                env.rustFuture,
                                env.waker,
                                env.selfAsCStructure,
                                polledResult,
                                _status
                            )
                        }

                        {# If the future ready? -#}
                        if (isReady) {
                            {# Resume the continuation with the lifted value if any, `Unit` otherwise  -#}
                            continuation.resume(
                            {% match func.return_type() -%}
                            {%- when Some with (return_type) -%}
                                {{ return_type|lift_fn}}(polledResult.getValue())
                            {%- when None -%}
                                Unit
                            {%- endmatch %}
                            )

                            {# Clean up the environment and the future -#}
                            _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.remove(hash)
                            rustCall() { _status ->
                                _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_drop(env.rustFuture, _status)
                            }
                        }
                    } catch (exception: Exception) {
                        {# Resume the continuation with the caught exception -#}
                        continuation.resumeWithException(exception)

                        {# Clean up the environment and the future -#}
                        _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.remove(hash)
                        rustCall() { _status ->
                            _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}_drop(env.rustFuture, _status)
                        }
                    }
                }
            }
        }
    }

    {# Declare the `result` variable that will receive the result of the future -#}
    val result: {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}Unit{% endmatch %}

    {# Get the coroutine scope -#}
    coroutineScope {
        {# Suspend the coroutine, and get a continuation -#}
        result = suspendCoroutine<
            {%- match func.return_type() %}
            {%- when Some with (return_type) -%}
                {{ return_type|type_name }}
            {%- when None -%}
                Unit
            {%- endmatch -%}
        > { continuation ->
            {# Create the future -#}
            val rustFuture = {% if is_meth -%}
                callWithPointer {
                    {% call to_ffi_call_with_prefix("it", func) %}
                }
            {%- else -%}
                {%- call to_ffi_call(func) -%}
            {%- endif %}

            {# Create the waker environment -#}
            val env = RustFutureWakerEnvironment(rustFuture, continuation, Waker(), RustFutureWakerEnvironmentCStructure(), this)
            val envHash = env.hashCode()
            env.selfAsCStructure.hash = envHash

            _UniFFILib.FUTURE_WAKER_ENVIRONMENTS.put(envHash, env)

            {# Call the waker to schedule a poll -#}
            env.waker.callback(env.selfAsCStructure)
        }
    }

    {# We have a result if no exception caught, let's return it! -#}
    return result
}
{%- endmacro -%}
