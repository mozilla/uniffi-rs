{%- if self.include_once_check("CallbackInterfaceRuntime.swift") %}{%- include "CallbackInterfaceRuntime.swift" %}{%- endif %}
{%- let trait_impl=format!("UniffiCallbackInterface{}", name) %}

// Put the implementation in a struct so we don't pollute the top-level namespace
fileprivate struct {{ trait_impl }} {

    // Create the VTable using a series of closures.
    // Swift automatically converts these into C callback functions.
    static var vtable: {{ vtable|ffi_type_name }} = {{ vtable|ffi_type_name }}(
        {%- for (ffi_callback, meth) in vtable_methods %}
        {{ meth.name()|fn_name }}: { (
            {%- for arg in ffi_callback.arguments() %}
            {{ arg.name()|var_name }}: {{ arg.type_().borrow()|ffi_type_name }},
            {%- endfor -%}
            {%- if ffi_callback.has_rust_call_status_arg() %}
            uniffiCallStatus: UnsafeMutablePointer<RustCallStatus>
            {%- endif %}
        ) in
            let uniffiObj: {{ type_name }}
            do {
                try uniffiObj = {{ ffi_converter_name }}.handleMap.get(handle: uniffiHandle)
            } catch {
                uniffiCallStatus.pointee.code = CALL_UNEXPECTED_ERROR
                uniffiCallStatus.pointee.errorBuf = {{ Type::String.borrow()|lower_fn }}("Callback handle map error: \(error)")
                return
            }
            let makeCall = { {% if meth.throws() %}try {% endif %}uniffiObj.{{ meth.name()|fn_name }}(
                {%- for arg in meth.arguments() %}
                {% if !config.omit_argument_labels() %} {{ arg.name()|arg_name }}: {% endif %}try {{ arg|lift_fn }}({{ arg.name()|var_name }}){% if !loop.last %},{% endif %}
                {%- endfor %}
            ) }
            {% match meth.return_type() %}
            {%- when Some(t) %}
            let writeReturn = { uniffiOutReturn.pointee = {{ t|lower_fn }}($0) }
            {%- when None %}
            let writeReturn = { () }
            {%- endmatch %}

            {%- match meth.throws_type() %}
            {%- when None %}
            uniffiTraitInterfaceCall(
                callStatus: uniffiCallStatus,
                makeCall: makeCall,
                writeReturn: writeReturn
            )
            {%- when Some(error_type) %}
            uniffiTraitInterfaceCallWithError(
                callStatus: uniffiCallStatus,
                makeCall: makeCall,
                writeReturn: writeReturn,
                lowerError: {{ error_type|lower_fn }}
            )
            {%- endmatch %}
        },
        {%- endfor %}
        uniffiFree: { (uniffiHandle: UInt64) -> () in
            let result = try? {{ ffi_converter_name }}.handleMap.remove(handle: uniffiHandle)
            if result == nil {
                print("Uniffi callback interface {{ name }}: handle missing in uniffiFree")
            }
        }
    )
}

private func {{ callback_init }}() {
    {{ ffi_init_callback.name() }}(&{{ trait_impl }}.vtable)
}
