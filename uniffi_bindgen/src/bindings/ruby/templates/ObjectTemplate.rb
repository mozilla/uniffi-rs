class {{ obj.name()|class_name_rb }}

  # A private helper for initializing instances of the class from a raw handle,
  # bypassing any initialization logic and ensuring they are GC'd properly.
  def self.uniffi_allocate(handle)
    inst = allocate
    inst.instance_variable_set :@handle, handle
    ObjectSpace.define_finalizer(inst, uniffi_define_finalizer_by_handle(handle, inst.object_id))
    return inst
  end

  # A private helper for registering an object finalizer.
  # N.B. it's important that this does not capture a reference
  # to the actual instance, only its underlying handle.
  def self.uniffi_define_finalizer_by_handle(handle, object_id)
    Proc.new do |_id|
      {{ ci.namespace()|class_name_rb }}.rust_call(
        :{{ obj.ffi_object_free().name() }},
        handle
      )
    end
  end

  # A private helper for lowering instances into a raw handle.
  # This does an explicit typecheck, because accidentally lowering a different type of
  # object in a place where this type is expected, could lead to memory unsafety.
  def self.uniffi_check_lower(inst)
    if not inst.is_a? self
      raise TypeError.new "Expected a {{ obj.name()|class_name_rb }} instance, got #{inst}"
    end
  end

  def uniffi_clone_handle()
    return {{ ci.namespace()|class_name_rb }}.rust_call(
      :{{ obj.ffi_object_clone().name() }},
      @handle
    )
  end

  def self.uniffi_lower(inst)
    return inst.uniffi_clone_handle()
  end

  {%- match obj.primary_constructor() %}
  {%- when Some with (cons) %}
  def initialize({% call rb::arg_list_decl(cons) -%})
    {%- call rb::setup_args_extra_indent(cons) %}
    handle = {% call rb::to_ffi_call(cons) %}
    @handle = handle
    ObjectSpace.define_finalizer(self, self.class.uniffi_define_finalizer_by_handle(handle, self.object_id))
  end
  {%- when None %}
  {%- endmatch %}

  {% for cons in obj.alternate_constructors() -%}
  def self.{{ cons.name()|fn_name_rb }}({% call rb::arg_list_decl(cons) %})
    {%- call rb::setup_args_extra_indent(cons) %}
    # Call the (fallible) function before creating any half-baked object instances.
    # Lightly yucky way to bypass the usual "initialize" logic
    # and just create a new instance with the required handle.
    return uniffi_allocate({% call rb::to_ffi_call(cons) %})
  end
  {% endfor %}

  {% for meth in obj.methods() -%}
  {%- match meth.return_type() -%}

  {%- when Some with (return_type) -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %})
    {%- call rb::setup_args_extra_indent(meth) %}
    result = {% call rb::to_ffi_call_with_prefix("uniffi_clone_handle()", meth) %}
    return {{ "result"|lift_rb(return_type) }}
  end

  {%- when None -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %})
      {%- call rb::setup_args_extra_indent(meth) %}
      {% call rb::to_ffi_call_with_prefix("uniffi_clone_handle()", meth) %}
  end
  {% endmatch %}
  {% endfor %}
end
