{#
// Template used to generate mock function.
// if is_alternate_constructor is true, then it generates a class function.
#}

    // MARK: - {{ meth.name()|fn_name }}

    {# // CallsCount -#}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}CallsCount = 0
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}Called: Bool {
        return {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}CallsCount > 0
    }
    {% if meth.arguments().len() > 0 -%}
    {# // ReceivedInvocations -#}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}ReceivedInvocations: [
    {%- if meth.arguments().len() > 1 -%}
    ({% call swift::arg_list_decl(meth) %})
    {%- else -%}
    {{ meth.arguments()[0]|type_name }}
    {%- endif -%}
    ] = []
    {# // ReceivedArguments -#}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}
    {%- if meth.arguments().len() > 1 -%}
    ReceivedArguments: ({% call swift::arg_list_decl(meth) %})?
    {%- else -%}
    ReceivedArgument: {{ meth.arguments()[0]|type_name }}{% if meth.arguments()[0]|type_is_optional == false %}?{% endif %}
    {% endif %}
    {# // ThrowableError -#}
    {%- endif %}
    {%- if meth.throws() -%}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}ThrowableError: Error?
    {%- endif %}
    {# // Closure -#}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) -%}Closure: ((
        {%- for arg in meth.arguments() -%}
        {{ arg|type_name }}
        {%- if !loop.last %}, {% endif -%}
        {%- endfor -%}
    ) {% call swift::throws(meth) -%} -> {% match meth.return_type() %}{% when Some with (return_type) %}{{ return_type|type_name }}{% when None %}Void{% endmatch %})?
    {# // ReturnValue -#}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}
    public {% if is_alternate_constructor %}static {% endif -%} var {% call swift::mock_var_prefix(meth) %}ReturnValue: {{ return_type|type_name }}{% if return_type|type_is_optional == false %}!{% endif %}
    {%- when None -%}
    {%- endmatch %}
    {# // Documentation -#}
    {% call swift::docstring(meth, 4) %}
    {# // Implementation -#}
    {%- if is_alternate_constructor -%}
    public override class func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ impl_class_name }} {
    {%- else -%}
    public override func {{ meth.name()|fn_name }}({%- call swift::arg_list_decl(meth) -%}) 
        {%- if meth.is_async() %} async{% endif %} {% call swift::throws(meth) %}
        {%- match meth.return_type() %}{% when Some with (return_type) -%} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
    {%- endif %}
        {# // Check for throwable error -#}
        {% if meth.throws() -%}
        if let error = {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) -%}ThrowableError {
            throw error
        }
        {%- endif %}
        {% if meth.arguments().len() > 0 -%}
        {# // Update received arguments -#}
        {%- if meth.arguments().len() > 1 -%}
        {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}ReceivedArguments = (
            {%- for arg in meth.arguments() -%}
            {{ arg.name()|var_name }}: {{ arg.name()|var_name }}
            {%- if !loop.last %}, {% endif -%}
            {%- endfor -%}
        )
        {% else %}
        {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}ReceivedArgument = {{ meth.arguments()[0].name()|var_name }}
        {% endif %}
        {# // Update received invocations -#}
        {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}ReceivedInvocations.append(
            {%- if meth.arguments().len() > 1 -%}
            (
                {%- for arg in meth.arguments() -%}
                {{ arg.name()|var_name }}: {{ arg.name()|var_name }}
                {%- if !loop.last %}, {% endif -%}
                {%- endfor -%}
            )
            {%- else -%}
            {{ meth.arguments()[0].name()|var_name }}
            {%- endif -%}
        )
        {% endif %}
        {# // Update calls count -#}
        {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}CallsCount += 1
        {% match meth.return_type() -%}
        {%- when Some with (return_type) %}
        {# // Check for closure -#}
        if let closure = {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}Closure {            
            return {% if meth.throws() %}try {% endif -%} closure(
                {%- for arg in meth.arguments() -%}
                {{ arg.name()|var_name }}
                {%- if !loop.last %}, {% endif -%}
                {%- endfor -%}
            )
        }
        {# // Returns the return value -#}
        return {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) -%}ReturnValue
        {%- when None %}
        {# // Check for closure -#}
        if let closure = {% if is_alternate_constructor %}Self.{% endif %}{% call swift::mock_var_prefix(meth) %}Closure {
            {% if meth.throws() %}try {% endif -%}
            closure(
                {%- for arg in meth.arguments() -%}
                {{ arg.name()|var_name }}
                {%- if !loop.last %}, {% endif -%}
                {%- endfor -%}
            )
        }
        {%- endmatch %}
    }