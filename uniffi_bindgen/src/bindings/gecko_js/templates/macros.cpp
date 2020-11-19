{#/* Calls an FFI function. */ #}
{%- macro to_ffi_call(context, func) -%}
  {%- call to_ffi_call_head(context, func, "err", "loweredRetVal_") -%}
  {%- call _to_ffi_call_tail(context, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{#/* Calls an FFI function with an initial argument. */ #}
{%- macro to_ffi_call_with_prefix(context, prefix, func) %}

  if (!XRE_IsParentProcess() && !NS_IsMainThread()) {
    aRv.Throw(NS_ERROR_UNEXPECTED);
    return nullptr;
  }

  RefPtr<Promise> promise = Promise::Create(GetParentObject(), aRv);
  if (aRv.Failed()) {
    return nullptr;
  }

  RefPtr<nsISerialEventTarget> backgroundET = GetBackgroundTarget();
  mozilla::InvokeAsync(
      backgroundET, __func__,
      [mHandle=mHandle
        {%- let args = func.arguments() -%}
        {%- if !args.is_empty() %},{% endif -%}
        {%- for arg in args %}
        {{ arg.name() }}={{ arg.name()}}{%- if !loop.last %},{% endif -%}
        {%- endfor %}]() {
        if (XRE_IsParentProcess() && NS_IsMainThread()) {
          MOZ_CRASH("lambda called outside of parent process main thread");
        }

        {{ context.ffi_rusterror_type() }} err = {0, nullptr};
        const{% match func.ffi_func().return_type() %}
        {% when Some with(type_) %}{{ type_|type_ffi(context) }} loweredRetVal_ = {% else %}int8_t loweredRetVal_ = 0; // MozPromise doesn't support void, so we use a dummy boolean (sigh)
        {% endmatch %}{{ func.ffi_func().name() }}(
          {{ prefix }}
          {%- if !args.is_empty() %},{% endif -%}
          {%- for arg in args %}
          {{ arg.webidl_type()|lower_cpp(arg.name(), context) }}{%- if !loop.last %},{% endif -%}
          {%- endfor %}, &err);
  {%- call _to_ffi_call_tail(context, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function without handling errors or lifting the return
      value. Used in the implementation of `to_ffi_call`, and for
      constructors. */ #}
{%- macro to_ffi_call_head(context, func, error, result) %}
  {{ context.ffi_rusterror_type() }} {{ error }} = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi(context) }} {{ result }} ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {%- let args = func.arguments() -%}
    {%- for arg in args %}
    {{ arg.webidl_type()|lower_cpp(arg.name(), context) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    {% if !args.is_empty() %}, {% endif %}&{{ error }}
  );
{%- endmacro -%}

{# /* Handles errors and lifts the return value from an FFI function. */ #}
{%- macro _to_ffi_call_tail(context, func, err, result) %}
    if ({{ err }}.mCode) {
      {%- match func.cpp_throw_by() %}
      {%- when ThrowBy::ErrorResult with (rv) %}
      {# /* TODO: Improve error throwing. See https://github.com/mozilla/uniffi-rs/issues/295
          for details. XXXdmose code below is different now that we're async */ -#}
      return MozPromise< {% match func.ffi_func().return_type() %} {% when Some with (type_) %}{{ type_|type_ffi(context) }} {% else %} int8_t{% endmatch %}, {{ context.ffi_rusterror_type() }}, false>::CreateAndReject(std::move({{ err }}), __func__);
      {%- when ThrowBy::Assert %}
      {%- endmatch %}
    }

    return MozPromise<{% match func.ffi_func().return_type() %}{% when Some with (type_) %}{{ type_|type_ffi(context) }} {% else %}int8_t{% endmatch %}, {{ context.ffi_rusterror_type() }}, false>::CreateAndResolve(std::move({{ result }}), __func__);
  })->Then(GetCurrentSerialEventTarget(), __func__,
    [promise]({% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi(context) }} {% else %}const int8_t{% endmatch %} {{result}}) {
      /* resolve DOM promise */

      {%- match func.cpp_return_by() %}
      {%- when ReturnBy::OutParam with (name, type_) %}
      DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, name, context) }};
      MOZ_ASSERT(ok_);
      {%- when ReturnBy::Value with (type_) %}
      {{ type_|type_cpp(context) }} retVal_;
      DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, "retVal_", context) }};
      MOZ_ASSERT(ok_);
      promise->MaybeResolve(retVal_);
      {%- when ReturnBy::Void %}
      promise->MaybeResolveWithUndefined();
      {%- endmatch %}
    },
    [promise]({{ context.ffi_rusterror_type() }} err) {
      // XXX put the message into the error
      // (a la aRv.ThrowOperationError(nsDependentCString(err.mMessage))
      promise->MaybeReject(NS_ERROR_FAILURE);
    });
  return promise.forget(); // XXX i assume .forget is needed, check on this
  {%- endmacro -%}
