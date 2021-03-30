// For each Object definition, we assume the caller has provided an appropriately-shaped `struct T`
// with an `impl` for each method on the object. We create an `Arc<T>` for "safely" handing out
// references to these structs to foreign language code, and we provide a `pub extern "C"` function
// corresponding to each method.
//
// (Note that "safely" is in "scare quotes" - that's because we use functions on an `Arc` that
// that are inherently unsafe, but the code we generate is safe in practice.)
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the UDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.

{% if obj.uses_deprecated_threadsafe_attribute() %}
// We want to mark this as `deprecated` - long story short, the only way to
// sanely do this using `#[deprecated(..)]` is to generate a function with that
// attribute, then generate call to that function in the object constructors.
#[deprecated(
    since = "0.11.0",
    note = "The `[Threadsafe]` attribute on interfaces is now the default and its use is deprecated - you should upgrade \
            `{{ obj.name() }}` to remove the `[ThreadSafe]` attribute. \
            See https://github.com/mozilla/uniffi-rs/#thread-safety for more details"
)]
#[allow(non_snake_case)]
fn uniffi_note_threadsafe_deprecation_{{ obj.name() }}() {}
{% endif %}

{% let ffi_free = obj.ffi_object_free() -%}
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ ffi_free.name() }}(ptr: *const std::os::raw::c_void) {
    // We mustn't panic across the FFI, but also can't report it anywhere.
    // The best we can do it catch, warn and ignore.
    if let Err(e) = std::panic::catch_unwind(|| {
        assert!(!ptr.is_null());
        {#- turn it into an Arc and explicitly drop it. #}
        drop(unsafe { std::sync::Arc::from_raw(ptr as *const {{ obj.name() }}) })
    }) {
        uniffi::deps::log::error!("{{ ffi_free.name() }} panicked: {:?}", e);
    }
}

{%- for cons in obj.constructors() %}
    #[allow(clippy::all)]
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn {{ cons.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(cons.ffi_func()) %}) -> *const std::os::raw::c_void /* *const {{ obj.name() }} */ {
        uniffi::deps::log::debug!("{{ cons.ffi_func().name() }}");
        {% if obj.uses_deprecated_threadsafe_attribute() %}
        uniffi_note_threadsafe_deprecation_{{ obj.name() }}();
        {% endif %}

        // If the constructor does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_constructor_call(obj, cons) %}
    }
{%- endfor %}

{%- for meth in obj.methods() %}
    #[allow(clippy::all)]
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn {{ meth.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(meth.ffi_func()) %}
    ) -> {% call rs::return_type_func(meth) %} {
        uniffi::deps::log::debug!("{{ meth.ffi_func().name() }}");
        // If the method does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_method_call(obj, meth) %}
    }
{% endfor %}
