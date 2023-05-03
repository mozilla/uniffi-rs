{%- let obj = ci.get_object_definition(name).unwrap() %}

typedef {{ type_name }}Lowered = Pointer<Void>;
typedef {{ type_name }}Lifted = {{ type_name }};

class {{ type_name }} {
    final {{ type_name }}Lowered _pointer;
    final Api _api;
  
    late final _dropPtr = _api._lookup<NativeFunction<Void Function(Pointer, Pointer<RustCallStatus>)>>("{{ obj.ffi_object_free().name() }}");
    late final _drop = _dropPtr.asFunction<void Function(Pointer, Pointer<RustCallStatus>)>();
  
    {% for meth in obj.methods( )%}
        {% call dart::gen_ffi_signatures(meth) %}
    {% endfor %}


    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    {{ type_name }}._(this._api, this._pointer, {% call dart::arg_list_decl(cons) -%}) {% call dart::throws(cons) %};
    {%- when None %}
    {%- endmatch %}

{#
    void drop() {
        _drop(pointer);
    }
#}

    {% for cons in obj.alternate_constructors() %}

    {{ type_name }} {{ cons.name()|fn_name }}({% call dart::arg_list_decl(cons) %}) {% call dart::throws(cons) %} {
        return {{ type_name }}(unsafeFromRawPointer: {% call dart::to_ffi_call(cons) %});
    }

    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- if meth.is_async() %}

    {%- match meth.return_type() -%}

    {%- when Some with (return_type) %}
    Future<{{ return_type|type_name }}> {{ meth.name()|fn_name }}({%- call dart::arg_list_decl(meth) -%}) async {
    {% when None %}
    Future<void> {{ meth.name()|fn_name }}({%- call dart::arg_list_decl(meth) -%}) async {
    {%- endmatch -%}
        let future = {% call dart::to_ffi_call_with_prefix("_pointer", meth) %}

        return {% if meth.throws() -%}
            try await withCheckedThrowingContinuation
        {%- else -%}
            await withCheckedContinuation
        {%- endif -%}
        { continuation in
            let env = Unmanaged.passRetained(_UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env(rustyFuture: future, continuation: continuation))
            _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker(raw_env: env.toOpaque())
        }
    }

    {% else -%}

    {%- match meth.return_type() -%}

    {%- when Some with (return_type) %}

    {{ return_type|type_name }} {{ meth.name()|fn_name }}({% call dart::arg_list_decl(meth) %}) {
        return {{ return_type|lift_fn }}(_api, 
            {% call dart::to_ffi_call_with_prefix("_pointer", meth) %}
        );
    }

    {%- when None %}

    void {{ meth.name()|fn_name }}({% call dart::arg_list_decl(meth) %}) {
        {% call dart::to_ffi_call_with_prefix("_pointer", meth) %}
    }

    {%- endmatch -%}
    {%- endif -%}
    {% endfor %}
}

{% for meth in obj.methods() -%}
{%- if meth.is_async() -%}

fileprivate class _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env {
    var rustFuture: OpaquePointer
    var continuation: CheckedContinuation<{% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, {% if meth.throws() %}Error{% else %}Never{% endif %}>

    init(rustyFuture: OpaquePointer, continuation: CheckedContinuation<{% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}(){% endmatch %}, {% if meth.throws() %}Error{% else %}Never{% endif %}>) {
        self.rustFuture = rustyFuture
        self.continuation = continuation
    }

    deinit {
        try! rustCall {
            {{ meth.ffi_func().name() }}_drop(self.rustFuture, $0)
        }
    }
}

fileprivate func _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker(raw_env: UnsafeMutableRawPointer?) {
    Task {
        let env = Unmanaged<_UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_Env>.fromOpaque(raw_env!)
        let env_ref = env.takeUnretainedValue()
        let polledResult = {% match meth.ffi_func().return_type() -%}
        {%- when Some with (return_type) -%}
            UnsafeMutablePointer<{{ return_type|type_ffi_lowered }}>.allocate(capacity: 1)
        {%- when None -%}
            UnsafeMutableRawPointer.allocate(byteCount: 0, alignment: 0)
        {%- endmatch %}
        {% if meth.throws() -%}do {
        {%- endif %}

        let isReady = {% match meth.throws_type() -%}
        {%- when Some with (error) -%}
            try rustCallWithError({{ error|ffi_converter_name }}.self) {
        {%- when None -%}
            try! rustCall() {
        {%- endmatch %}
            {{ meth.ffi_func().name() }}_poll(
                env_ref.rustFuture,
                _UniFFI_{{ obj.name() }}_{{ meth.name()|class_name }}_waker,
                env.toOpaque(),
                polledResult,
                $0
            )
        }

        if isReady {
            env_ref.continuation.resume(returning: {% match meth.return_type() -%}
            {%- when Some with (return_type) -%}
                try! {{ return_type|lift_fn }}(polledResult.move())
            {%- when None -%}
                ()
            {%- endmatch -%}
            )
            polledResult.deallocate()
            env.release()
        }
        {%- if meth.throws() %}
        } catch {
            env_ref.continuation.resume(throwing: error)
            polledResult.deallocate()
            env.release()
        }
        {%- endif %}
    }
}

{% endif -%}
{%- endfor %}

class {{ ffi_converter_name }} {
{#
    {{ type_name }}? read(from buf: inout (data: Data, offset: Data.Index)) {
        let v: UInt64 = try readInt(&buf)
        // The Rust code won't compile if a pointer won't fit in a UInt64.
        // We have to go via `UInt` because that's the thing that's the size of a pointer.
        let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: v))
        if (ptr == nil) {
            throw UniffiInternalError.unexpectedNullPointer
        }
        return try lift(ptr!)
    }

    void write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        // This fiddling is because `Int` is the thing that's the same size as a pointer.
        // The Rust code won't compile if a pointer won't fit in a `UInt64`.
        writeInt(&buf, UInt64(bitPattern: Int64(Int(bitPattern: lower(value)))))
    }
#}
    static {{ type_name }}Lifted? lift(Api api, {{ type_name }}Lowered pointer) {
        return {{ type_name}}._(api, pointer);
    }

    static {{ type_name }}Lowered lower({{ type_name }}Lifted value) {
        return value._pointer;
    }
}

{#
We always write these public functions just in case the struct is used as
an external type by another crate.
#}
{{ type_name }}? {{ ffi_converter_name }}_lift(Api api, {{ type_name }}Lowered pointer) {
    return {{ ffi_converter_name }}.lift(api, pointer);
}

{{ type_name }}Lowered? {{ ffi_converter_name }}_lower({{ type_name }}Lifted value) {
    return {{ ffi_converter_name }}.lower(value);
}
