extern "C" {

struct {{ context.ffi_rustbuffer_type() }} {
  int32_t mCapacity;
  int32_t mLen;
  uint8_t* mData;

  // Ref https://github.com/mozilla/uniffi-rs/issues/334 re mPadding workaround
  int64_t mPadding;
};

struct {{ context.ffi_foreignbytes_type() }} {
  int32_t mLen;
  const uint8_t* mData;

  // Ref https://github.com/mozilla/uniffi-rs/issues/334 re padding workarounds
  int64_t mPadding;
  int32_t mPadding2;
};

struct {{ context.ffi_rustcallstatus_type() }} {
  int32_t mCode;
  {{ context.ffi_rustbuffer_type() }} mErrorBuf;
};

{% for func in ci.iter_ffi_function_definitions() -%}
{%- match func.return_type() -%}
{%- when Some with (type_) %}
{{ type_|type_ffi(context) }}
{% when None %}
void
{%- endmatch %}
{{ func.name() }}(
    {%- for arg in func.arguments() %}
    {{ arg.type_()|type_ffi(context) }} {{ arg.name() -}}{%- if loop.last -%}{%- else -%},{%- endif -%}
    {%- endfor %}
    {%- if func.arguments().len() > 0 %},{% endif %}
    {{ context.ffi_rustcallstatus_type() }}* uniffi_out_status
);

{% endfor -%}

}  // extern "C"
