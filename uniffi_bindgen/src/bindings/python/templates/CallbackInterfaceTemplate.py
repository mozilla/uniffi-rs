{%- let cbi = ci.get_callback_interface_definition(id).unwrap() %}
{%- let foreign_callback = format!("foreignCallback{}", canonical_type_name) %}

{% if self.include_once_check("CallbackInterfaceRuntime.py") %}{% include "CallbackInterfaceRuntime.py" %}{% endif %}

# Declaration and FfiConverters for {{ type_name }} Callback Interface

class {{ type_name }}:
    {% for meth in cbi.methods() -%}
    def {{ meth.name()|fn_name }}({% call py::arg_list_decl(meth) %}):
        raise NotImplementedError

    {% endfor %}

def py_{{ foreign_callback }}(handle, method, args, buf_ptr):
    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name %}
    def {{ method_name }}(python_callback, args, buf_ptr):
        {#- Unpacking args from the RustBuffer #}
        def makeCall():
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            with args.contents.readWithStream() as buf:
                return python_callback.{{ meth.name()|fn_name }}(
                    {% for arg in meth.arguments() -%}
                    {{ arg|read_fn }}(buf)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            {%- else %}
            return python_callback.{{ meth.name()|fn_name }}()
            {%- endif %}

        def makeCallAndHandleReturn():
            {%- match meth.return_type() %}
            {%- when Some(return_type) %}
            rval = makeCall()
            with RustBuffer.allocWithBuilder() as builder:
                {{ return_type|write_fn }}(rval, builder)
                buf_ptr[0] = builder.finalize()
            {%- when None %}
            makeCall()
            {%- endmatch %}
            return 1

        {%- match meth.throws_type() %}
        {%- when None %}
        return makeCallAndHandleReturn()
        {%- when Some(err) %}
        try:
            return makeCallAndHandleReturn()
        except {{ err|type_name }} as e:
            # Catch errors declared in the UDL file
            with RustBuffer.allocWithBuilder() as builder:
                {{ err|write_fn }}(e, builder)
                buf_ptr[0] = builder.finalize()
            return -2
        {%- endmatch %}

    {% endfor %}

    cb = {{ ffi_converter_name }}.lift(handle)
    if not cb:
        raise InternalError("No callback in handlemap; this is a Uniffi bug")

    if method == IDX_CALLBACK_FREE:
        {{ ffi_converter_name }}.drop(handle)
        # No return value.
        # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
        return 0

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
    if method == {{ loop.index }}:
        # Call the method and handle any errors
        # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs` for details
        try:
            return {{ method_name }}(cb, args, buf_ptr)
        except BaseException as e:
            # Catch unexpected errors
            try:
                # Try to serialize the exception into a String
                buf_ptr[0] = {{ Type::String.borrow()|lower_fn }}(repr(e))
            except:
                # If that fails, just give up
                pass
            return -1
    {% endfor %}

    # This should never happen, because an out of bounds method index won't
    # ever be used. Once we can catch errors, we should return an InternalException.
    # https://github.com/mozilla/uniffi-rs/issues/351

    # An unexpected error happened.
    # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
    return -1

# We need to keep this function reference alive:
# if they get GC'd while in use then UniFFI internals could attempt to call a function
# that is in freed memory.
# That would be...uh...bad. Yeah, that's the word. Bad.
{{ foreign_callback }} = FOREIGN_CALLBACK_T(py_{{ foreign_callback }})
rust_call(lambda err: _UniFFILib.{{ cbi.ffi_init_callback().name() }}({{ foreign_callback }}, err))

# The FfiConverter which transforms the Callbacks in to Handles to pass to Rust.
{{ ffi_converter_name }} = FfiConverterCallbackInterface({{ foreign_callback }})
