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
NS_IMPL_CYCLE_COLLECTION_WRAPPERCACHE({{ obj.name()|class_name_cpp(context) }}, mGlobal)
NS_IMPL_CYCLE_COLLECTING_ADDREF({{ obj.name()|class_name_cpp(context) }})
NS_IMPL_CYCLE_COLLECTING_RELEASE({{ obj.name()|class_name_cpp(context) }})
NS_INTERFACE_MAP_BEGIN_CYCLE_COLLECTION({{ obj.name()|class_name_cpp(context) }})
  NS_WRAPPERCACHE_INTERFACE_MAP_ENTRY
  NS_INTERFACE_MAP_ENTRY(nsISupports)
NS_INTERFACE_MAP_END

{{ obj.name()|class_name_cpp(context) }}::{{ obj.name()|class_name_cpp(context) }}(
  nsIGlobalObject* aGlobal,
  uint64_t aHandle
) : mGlobal(aGlobal), mHandle(aHandle) {}

{{ obj.name()|class_name_cpp(context) }}::~{{ obj.name()|class_name_cpp(context) }}() {
  {{ context.ffi_rusterror_type() }} err = {0, nullptr};
  {{ obj.ffi_object_free().name() }}(mHandle, &err);
  MOZ_ASSERT(!err.mCode);
}

JSObject* {{ obj.name()|class_name_cpp(context) }}::WrapObject(
  JSContext* aCx,
  JS::Handle<JSObject*> aGivenProto
) {
  return dom::{{ obj.name()|class_name_cpp(context) }}_Binding::Wrap(aCx, this, aGivenProto);
}

{%- for cons in obj.constructors() %}

/* static */
already_AddRefed<{{ obj.name()|class_name_cpp(context) }}> {{ obj.name()|class_name_cpp(context) }}::Constructor(
  {%- for arg in cons.cpp_arguments() %}
  {{ arg|arg_type_cpp(context) }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
  {%- endfor %}
) {
  {%- call cpp::to_ffi_call_head(context, cons, "err", "handle") %}
  if (err.mCode) {
    {%- match cons.cpp_throw_by() %}
    {%- when ThrowBy::ErrorResult with (rv) %}
    {{ rv }}.ThrowOperationError(nsDependentCString(err.mMessage));
    {%- when ThrowBy::Assert %}
    MOZ_ASSERT(false);
    {%- endmatch %}
    return nullptr;
  }
  nsCOMPtr<nsIGlobalObject> global = do_QueryInterface(aGlobal.GetAsSupports());
  auto result = MakeRefPtr<{{ obj.name()|class_name_cpp(context) }}>(global, handle);
  return result.forget();
}
{%- endfor %}

{%- for meth in obj.methods() %}
{% if meth.is_static() %}
MOZ_STATIC_ASSERT(false, "Sorry the gecko-js backend does not yet support static methods");
{% endif %}

{% match meth.cpp_return_type() %}{% when Some with (type_) %}{{ type_|ret_type_cpp(context) }}{% else %}void{% endmatch %} {{ obj.name()|class_name_cpp(context) }}::{{ meth.name()|fn_name_cpp }}(
  {%- for arg in meth.cpp_arguments() %}
  {{ arg|arg_type_cpp(context) }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
  {%- endfor %}
) {
  {%- call cpp::to_ffi_call_with_prefix(context, "mHandle", meth) %}
}
{%- endfor %}

}  // namespace dom
}  // namespace mozilla
