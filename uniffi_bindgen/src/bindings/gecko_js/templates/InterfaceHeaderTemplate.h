// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

{% import "macros.cpp" as cpp %}

#ifndef mozilla_dom_{{ obj.name()|header_name_cpp }}
#define mozilla_dom_{{ obj.name()|header_name_cpp }}

#include "jsapi.h"
#include "nsCOMPtr.h"
#include "nsIGlobalObject.h"
#include "nsWrapperCache.h"

#include "mozilla/RefPtr.h"

#include "mozilla/dom/{{ context.namespace()|class_name_webidl(context) }}Binding.h"

namespace mozilla {
namespace dom {

class {{ obj.name()|class_name_cpp }} final : public nsISupports, public nsWrapperCache {
 public:
  NS_DECL_CYCLE_COLLECTING_ISUPPORTS
  NS_DECL_CYCLE_COLLECTION_SCRIPT_HOLDER_CLASS({{ obj.name()|class_name_cpp }})

  {{ obj.name()|class_name_cpp }}(nsIGlobalObject* aGlobal, uint64_t aHandle);

  JSObject* WrapObject(JSContext* aCx,
                       JS::Handle<JSObject*> aGivenProto) override;

  nsIGlobalObject* GetParentObject() const { return mGlobal; }

  {%- for cons in obj.constructors() %}

  static already_AddRefed<{{ obj.name()|class_name_cpp }}> Constructor(
    {%- for arg in cons.binding_arguments() %}
    {{ arg|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
    {%- endfor %}
  );
  {%- endfor %}

  {%- for meth in obj.methods() %}

  {% match meth.binding_return_type() %}{% when Some with (type_) %}{{ type_|ret_type_cpp }}{% else %}void{% endmatch %} {{ meth.name()|fn_name_cpp }}(
    {%- for arg in meth.binding_arguments() %}
    {{ arg|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
    {%- endfor %}
  );
  {%- endfor %}

 private:
  ~{{ obj.name()|class_name_cpp }}();

  nsCOMPtr<nsIGlobalObject> mGlobal;
  uint64_t mHandle;
};

}  // namespace dom
}  // namespace mozilla

#endif  // mozilla_dom_{{ obj.name()|header_name_cpp }}
