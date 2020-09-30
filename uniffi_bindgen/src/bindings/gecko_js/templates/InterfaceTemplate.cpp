// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

{% import "macros.cpp" as cpp %}

#include "mozilla/dom/{{ obj.name()|header_name_cpp(context) }}.h"
#include "mozilla/dom/{{ context.namespace()|header_name_cpp(context) }}Shared.h"

namespace mozilla {
namespace dom {

// Cycle collection boilerplate for our interface implementation. `mGlobal` is
// the only member that needs to be cycle-collected; if we ever add any JS
// object members or other interfaces to the class, those should be collected,
// too.
NS_IMPL_CYCLE_COLLECTION_WRAPPERCACHE({{ obj.name()|type_name(context)|class_name_cpp }}, mGlobal)
NS_IMPL_CYCLE_COLLECTING_ADDREF({{ obj.name()|type_name(context)|class_name_cpp }})
NS_IMPL_CYCLE_COLLECTING_RELEASE({{ obj.name()|type_name(context)|class_name_cpp }})
NS_INTERFACE_MAP_BEGIN_CYCLE_COLLECTION({{ obj.name()|type_name(context)|class_name_cpp }})
  NS_WRAPPERCACHE_INTERFACE_MAP_ENTRY
  NS_INTERFACE_MAP_ENTRY(nsISupports)
NS_INTERFACE_MAP_END

{{ obj.name()|type_name(context)|class_name_cpp }}::{{ obj.name()|type_name(context)|class_name_cpp }}(
  nsIGlobalObject* aGlobal,
  uint64_t aHandle
) : mGlobal(aGlobal), mHandle(aHandle) {}

{{ obj.name()|type_name(context)|class_name_cpp }}::~{{ obj.name()|type_name(context)|class_name_cpp }}() {
  RustError err = {0, nullptr};
  {{ obj.ffi_object_free().name() }}(mHandle, &err);
  MOZ_ASSERT(!err.mCode);
}

JSObject* {{ obj.name()|type_name(context)|class_name_cpp }}::WrapObject(
  JSContext* aCx,
  JS::Handle<JSObject*> aGivenProto
) {
  return dom::{{ obj.name()|type_name(context)|class_name_cpp }}_Binding::Wrap(aCx, this, aGivenProto);
}

{%- for cons in obj.constructors() %}

/* static */
already_AddRefed<{{ obj.name()|type_name(context)|class_name_cpp }}> {{ obj.name()|type_name(context)|class_name_cpp }}::Constructor(
  {%- for arg in cons.binding_arguments() %}
  {{ arg|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
  {%- endfor %}
) {
  {%- call cpp::to_ffi_call_head(context, cons, "err", "handle") %}
  if (err.mCode) {
    {%- match cons.throw_by() %}
    {%- when ThrowBy::ErrorResult with (rv) %}
    {{ rv }}.ThrowOperationError(err.mMessage);
    {%- when ThrowBy::Assert %}
    MOZ_ASSERT(false);
    {%- endmatch %}
    return nullptr;
  }
  nsCOMPtr<nsIGlobalObject> global = do_QueryInterface(aGlobal.GetAsSupports());
  auto result = MakeRefPtr<{{ obj.name()|type_name(context)|class_name_cpp }}>(global, handle);
  return result.forget();
}
{%- endfor %}

{%- for meth in obj.methods() %}

{% match meth.binding_return_type() %}{% when Some with (type_) %}{{ type_|ret_type_cpp }}{% else %}void{% endmatch %} {{ obj.name()|type_name(context)|class_name_cpp }}::{{ meth.name()|fn_name_cpp }}(
  {%- for arg in meth.binding_arguments() %}
  {{ arg|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
  {%- endfor %}
) {
  {%- call cpp::to_ffi_call_with_prefix(context, "mHandle", meth) %}
}
{%- endfor %}

}  // namespace dom
}  // namespace mozilla
