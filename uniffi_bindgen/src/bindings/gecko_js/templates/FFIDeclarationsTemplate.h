extern "C" {

struct {{ context.ffi_rustbuffer_type() }} {
  int32_t mCapacity;
  int32_t mLen;
  uint8_t* mData;
  uint64_t mPadding;
};

struct {{ context.ffi_foreignbytes_type() }} {
  int32_t mLen;
  const uint8_t* mData;
  int64_t mPadding;
  int32_t mPadding2;
};

struct {{ context.ffi_rusterror_type() }} {
  int32_t mCode;
  char* mMessage;
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
    {{ context.ffi_rusterror_type() }}* uniffi_out_err
);

{% endfor -%}

}  // extern "C"
