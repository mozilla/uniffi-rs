class {{ obj.name()|class_name_rb }}{% if ci.is_name_used_as_error(obj.name()) %} < StandardError{% endif %}

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
      ::{{ ci.namespace()|class_name_rb }}.rust_call(
        :{{ obj.ffi_object_free().name() }},
        handle
      )
    end
  end

  {%- if obj.has_callback_interface() %}
  # HandleMap for trait interface: stores Ruby-native objects keyed by odd handles.
  @uniffi_handle_map = UniffiHandleMap.new

  class << self
    attr_reader :uniffi_handle_map
  end

  # For trait interfaces: check that the object is either a Rust-backed instance or
  # responds to the required interface methods.
  def self.uniffi_check_lower(inst)
    {%- for method in obj.methods() %}
    if !inst.respond_to?(:{{ method.name()|fn_name_rb }})
      raise TypeError.new "Expected a {{ obj.name()|class_name_rb }} instance or an object implementing the interface, got #{inst}"
    end
    {%- endfor %}
  end

  def uniffi_clone_handle
    return ::{{ci.namespace()|class_name_rb }}.rust_call(
      :{{ obj.ffi_object_clone().name() }},
      @handle
    )
  end

  # For trait interfaces: lowering distinguishes between Rust-backed and Ruby-native instances.
  def self.uniffi_lower(inst)
    if inst.is_a?(self) && inst.instance_variable_defined?(:@handle)
      inst.uniffi_clone_handle()
    else
      @uniffi_handle_map.insert(inst)
    end
  end

  # For trait interfaces: lifting distinguishes even handles (Rust) from odd handles (Ruby).
  def self.uniffi_lift(handle)
    if (handle & 1) == 0
      uniffi_allocate handle
    else
      @uniffi_handle_map.remove handle
    end
  end
  
  {%- else %}
  # A private helper for lowering instances into a raw handle.
  # This does an explicit typecheck, because accidentally lowering a different type of
  # object in a place where this type is expected, could lead to memory unsafety.
  def self.uniffi_check_lower(inst)
    if !inst.is_a? self
      raise TypeError.new "Expected a {{ obj.name()|class_name_rb }} instance, got #{inst}"
    end
  end

  def uniffi_clone_handle()
    return ::{{ ci.namespace()|class_name_rb }}.rust_call(
      :{{ obj.ffi_object_clone().name() }},
      @handle
    )
  end

  def self.uniffi_lower(inst)
    return inst.uniffi_clone_handle()
  end

  def self.uniffi_lift(handle)
    uniffi_allocate handle
  end
  {%- endif %}

  {%- match obj.primary_constructor() %}
  {%- when Some with (cons) %}
  {%- if cons.is_async() %}
  def initialize({% call rb::arg_list_decl(cons) %}{% endcall %})
    {%- call rb::setup_args_extra_indent(cons) %}{% endcall %}
    handle = {% call rb::to_ffi_call_async_constructor(cons) %}{% endcall %}
    @handle = handle
    ObjectSpace.define_finalizer(self, self.class.uniffi_define_finalizer_by_handle(handle, self.object_id))
  end
  {%- else %}
  def initialize({% call rb::arg_list_decl(cons) %}{% endcall -%})
    {%- call rb::setup_args_extra_indent(cons) %}{% endcall %}
    handle = {% call rb::to_ffi_call(cons) %}{% endcall %}
    @handle = handle
    ObjectSpace.define_finalizer(self, self.class.uniffi_define_finalizer_by_handle(handle, self.object_id))
  end
  {%- endif %}
  {%- when None %}
  {%- endmatch %}

  {% for cons in obj.alternate_constructors() -%}
  {%- if cons.is_async() %}
  def self.{{ cons.name()|fn_name_rb }}({% call rb::arg_list_decl(cons) %}{% endcall %})
    {%- call rb::setup_args_extra_indent(cons) %}{% endcall %}
    # Call the (fallible) async function before creating any half-baked object instances.
    return uniffi_allocate({% call rb::to_ffi_call_async_constructor(cons) %}{% endcall %})
  end
  {%- else %}
  def self.{{ cons.name()|fn_name_rb }}({% call rb::arg_list_decl(cons) %}{% endcall %})
    {%- call rb::setup_args_extra_indent(cons) %}{% endcall %}
    # Call the (fallible) function before creating any half-baked object instances.
    # Lightly yucky way to bypass the usual "initialize" logic
    # and just create a new instance with the required handle.
    return uniffi_allocate({% call rb::to_ffi_call(cons) %}{% endcall %})
  end
  {%- endif %}
  {% endfor %}

  {% for meth in obj.methods() -%}
  {%- if meth.is_async() %}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %}{% endcall %})
    {%- call rb::setup_args_extra_indent(meth) %}{% endcall %}
    {% call rb::to_ffi_call_with_prefix_async("uniffi_clone_handle()", meth) %}{% endcall %}
  end
  {%- else %}
  {%- match meth.return_type() -%}

  {%- when Some with (return_type) -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %}{% endcall %})
    {%- call rb::setup_args_extra_indent(meth) %}{% endcall %}
    result = {% call rb::to_ffi_call_with_prefix("uniffi_clone_handle()", meth) %}{% endcall %}
    return {{ "result"|lift_rb(return_type, config) }}
  end

  {%- when None -%}
  def {{ meth.name()|fn_name_rb }}({% call rb::arg_list_decl(meth) %}{% endcall %})
      {%- call rb::setup_args_extra_indent(meth) %}{% endcall %}
      {% call rb::to_ffi_call_with_prefix("uniffi_clone_handle()", meth) %}{% endcall %}
  end
  {% endmatch %}
  {%- endif %}
  {% endfor %}
  {%- let trait_methods = obj.uniffi_trait_methods() %}
  {%- include "UniffiTraitImpls.rb" %}
end

{%- if obj.has_callback_interface() %}
{# For trait interfaces, generate and register the vtable. #}
{%- let vtable = obj.vtable().expect("trait interface must have a vtable") %}
{%- let vtable_methods = obj.vtable_methods() %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{%- let cbi_name = obj.name() %}
{%- include "TraitInterfaceVtable.rb" %}
{%- endif %}
