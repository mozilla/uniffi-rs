# Put all the bits inside a class to keep the top-level namespace clean
class {{ vtable.name }}:
    # For each method, generate a callback function to pass to Rust
    {%- for VTableMethod { callable, name, ffi_type, default_return_value } in vtable.methods %}

    @{{ ffi_type }}
    def {{ name }}(
        uniffi_handle,
        {%- for arg in callable.arguments() %}
        {{ arg.name }},
        {%- endfor %}
        {%- if callable.is_sync() %}
        uniffi_out_return,
        uniffi_call_status_ptr
        {%- else %}
        uniffi_future_callback,
        uniffi_callback_data,
        uniffi_out_return,
        {%- endif %}
    ):
        uniffi_obj = {{ ffi_converter_name }}._handle_map.get(uniffi_handle)
        def make_call():
            return uniffi_obj.{{ name }}({% for arg in callable.arguments() %}{{ arg|lift_fn }}({{ arg.name }}), {% endfor %})

        {% match callable.async_data() %}
        {% when None %}
        {%- match callable.return_type() %}
        {%- when Some(return_type) %}
        def write_return_value(v):
            uniffi_out_return[0] = {{ return_type|lower_fn }}(v)
        {%- when None %}
        write_return_value = lambda v: None
        {%- endmatch %}

        {%- match callable.throws_type() %}
        {%- when None %}
        _uniffi_trait_interface_call(
                uniffi_call_status_ptr.contents,
                make_call,
                write_return_value,
        )
        {%- when Some(error) %}
        _uniffi_trait_interface_call_with_error(
                uniffi_call_status_ptr.contents,
                make_call,
                write_return_value,
                {{ error.type_name }},
                {{ error|lower_fn }},
        )
        {%- endmatch %}
        {% when Some(async_data) %}
        def handle_success(return_value):
            uniffi_future_callback(
                uniffi_callback_data,
                {{ async_data.foreign_future_result_type }}(
                    {%- if let Some(return_type) = callable.return_type() %}
                    {{ return_type|lower_fn }}(return_value),
                    {%- endif %}
                    _UniffiRustCallStatus.default()
                )
            )

        def handle_error(status_code, rust_buffer):
            uniffi_future_callback(
                uniffi_callback_data,
                {{ async_data.foreign_future_result_type }}(
                    {%- if callable.return_type().is_some() %}
                    {{ default_return_value }},
                    {%- endif %}
                    _UniffiRustCallStatus(status_code, rust_buffer),
                )
            )

        {%- match callable.throws_type() %}
        {%- when None %}
        uniffi_out_return[0] = _uniffi_trait_interface_call_async(make_call, handle_success, handle_error)
        {%- when Some(error) %}
        uniffi_out_return[0] = _uniffi_trait_interface_call_async_with_error(make_call, handle_success, handle_error, {{ error.type_name }}, {{ error|lower_fn }})
        {%- endmatch %}
        {%- endmatch %}
    {%- endfor %}

    @{{ globals.callback_interface_free_type }}
    def _uniffi_free(uniffi_handle):
        {{ ffi_converter_name }}._handle_map.remove(uniffi_handle)

    # Generate the FFI VTable.  This has a field for each callback interface method.
    _uniffi_vtable = {{ vtable.ffi_type }}(
        {%- for vmeth in vtable.methods %}
        {{ vmeth.name }},
        {%- endfor %}
        _uniffi_free
    )
    # Send Rust a pointer to the VTable.  Note: this means we need to keep the struct alive forever,
    # or else bad things will happen when Rust tries to access it.
    _UniffiLib.{{ vtable.ffi_init_callback }}(ctypes.byref(_uniffi_vtable))
