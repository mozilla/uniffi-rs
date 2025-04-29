# Put all the bits inside a class to keep the top-level namespace clean
class {{ trait_impl }}:
    # For each method, generate a callback function to pass to Rust
    {%- for meth in vtable.methods %}
    {%- let callable = meth.callable %}

    @{{ meth.ffi_type.type_name }}
    def {{ callable.name }}(
            uniffi_handle,
            {%- for arg in callable.arguments %}
            {{ arg.name }},
            {%- endfor -%}
            {%- if !callable.is_async %}
            uniffi_out_return,
            uniffi_call_status_ptr,
            {%- else %}
            uniffi_future_callback,
            uniffi_callback_data,
            uniffi_out_return,
            {%- endif %}
        ):
        uniffi_obj = {{ ffi_converter_name }}._handle_map.get(uniffi_handle)
        def make_call():
            args = ({% for arg in callable.arguments %}{{ arg.ty.ffi_converter_name }}.lift({{ arg.name }}), {% endfor %})
            method = uniffi_obj.{{ callable.name }}
            return method(*args)

        {%- match callable.async_data %}
        {%- when None %}
        {%- match callable.return_type.ty %}
        {%- when Some(return_type) %}
        def write_return_value(v):
            uniffi_out_return[0] = {{ return_type.ffi_converter_name }}.lower(v)
        {%- when None %}
        write_return_value = lambda v: None
        {%- endmatch %}

        {%- match callable.throws_type.ty %}
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
                {{ error.ffi_converter_name }}.lower,
        )
        {%- endmatch %}
        {%- when Some(async_data) %}
        def handle_success(return_value):
            uniffi_future_callback(
                uniffi_callback_data,
                {{ async_data.ffi_foreign_future_result.0 }}(
                    {%- if let Some(return_type) = callable.return_type.ty %}
                    {{ return_type.ffi_converter_name }}.lower(return_value),
                    {%- endif %}
                    _UniffiRustCallStatus.default()
                )
            )

        def handle_error(status_code, rust_buffer):
            uniffi_future_callback(
                uniffi_callback_data,
                {{ async_data.ffi_foreign_future_result.0 }}(
                    {%- match callable.return_type.ty %}
                    {%- when Some(return_type) %}
                    {{ meth.ffi_default_value }},
                    {%- when None %}
                    {%- endmatch %}
                    _UniffiRustCallStatus(status_code, rust_buffer),
                )
            )

        {%- match callable.throws_type.ty %}
        {%- when None %}
        uniffi_out_return[0] = _uniffi_trait_interface_call_async(make_call, handle_success, handle_error)
        {%- when Some(error) %}
        uniffi_out_return[0] = _uniffi_trait_interface_call_async_with_error(
            make_call,
            handle_success,
            handle_error,
            {{ error.type_name }},
            {{ error.ffi_converter_name }}.lower,
        )
        {%- endmatch %}
        {%- endmatch %}
    {%- endfor %}

    @{{ vtable.free_fn_type.0 }}
    def _uniffi_free(uniffi_handle):
        {{ ffi_converter_name }}._handle_map.remove(uniffi_handle)

    # Generate the FFI VTable.  This has a field for each callback interface method.
    _uniffi_vtable = {{ vtable.struct_type.type_name }}(
        {%- for meth in vtable.methods %}
        {{ meth.callable.name }},
        {%- endfor %}
        _uniffi_free
    )
    # Send Rust a pointer to the VTable.  Note: this means we need to keep the struct alive forever,
    # or else bad things will happen when Rust tries to access it.
    _UniffiLib.{{ vtable.init_fn.0 }}(ctypes.byref(_uniffi_vtable))
