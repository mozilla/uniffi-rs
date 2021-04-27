class {{ obj.name()|class_name_rb }}
  {%- match obj.primary_constructor() %}
  {%- when Some with (cons) %}
  def initialize({% call rb::arg_list_decl(cons) -%})
    {%- call rb::coerce_args_extra_indent(cons) %}
    handle = {% call rb::to_ffi_call(cons) %}
    ObjectSpace.define_finalizer(self, self.class.define_finalizer_by_handle(handle))
    @handle = handle
  end

  def {{ obj.name()|class_name_rb }}.define_finalizer_by_handle(handle)
    Proc.new do |_id|
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error(
        InternalError,
        :{{ obj.ffi_object_free().name() }},
        handle
      )
    end
  end
  {%- when None %}
  {%- endmatch %}

  {% for cons in obj.alternate_constructors() -%}
  def self.{{ cons.name()|fn_name_rb }}({% call rb::arg_list_decl(cons) %})
    {%- call rb::coerce_args_extra_indent(cons) %}
    # Call the (fallible) function before creating any half-baked object instances.
    # Lightly yucky way to bypass the usual "initialize" logic
    # and just create a new instance with the required handle.
    inst = allocate
    inst.instance_variable_set :@handle, {% call rb::to_ffi_call(cons) %}

    return inst
  end
  {% endfor %}

  {% for meth in obj.methods() -%}
  {%- match meth.return_type() -%}

  {%- when Some with (return_type) -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %})
    {%- call rb::coerce_args_extra_indent(meth) %}
    result = {% call rb::to_ffi_call_with_prefix("@handle", meth) %}
    return {{ "result"|lift_rb(return_type) }}
  end

  {%- when None -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %})
      {%- call rb::coerce_args_extra_indent(meth) %}
      {% call rb::to_ffi_call_with_prefix("@handle", meth) %}
  end
  {% endmatch %}
  {% endfor %}
end
