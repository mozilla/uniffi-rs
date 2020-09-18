extern "C" {

struct RustBuffer {
  int32_t mCapacity;
  int32_t mLen;
  uint8_t* mData;
};

struct ForeignBytes {
  int32_t mLen;
  const uint8_t* mData;
};

struct RustError {
  int32_t mCode;
  char* mMessage;
};

{% for func in ci.iter_ffi_function_definitions() -%}
{%- match func.return_type() -%}
{%- when Some with (type_) %}
{{ type_|type_ffi }}
{% when None %}
void
{%- endmatch %}
{{ func.name() }}(
    {%- for arg in func.arguments() %}
    {{ arg.type_()|type_ffi }} {{ arg.name() -}}{%- if loop.last -%}{%- else -%},{%- endif -%}
    {%- endfor %}
    {%- if func.arguments().len() > 0 %},{% endif %}
    RustError* uniffi_out_err
);

{% endfor -%}

}  // extern "C"
