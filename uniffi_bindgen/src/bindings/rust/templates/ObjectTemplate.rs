pub struct {{ obj.name()|class_name_rs }};

impl {{ obj.name()|class_name_rs }} {
  {%- match obj.primary_constructor() %}
  {%- when Some with (cons) %}
  pub fn new({% call rs::arg_list_decl(cons.arguments()) -%}) -> {% call rs::fallible_return_type(cons, "Self") %} {
    todo!()
  }

  {%- when None %}
  {%- endmatch %}

  {% for cons in obj.alternate_constructors() -%}
  pub fn {{ cons.name()|fn_name_rs }}({% call rs::arg_list_decl(cons.arguments()) %}) -> {% call rs::fallible_return_type(cons, "Self") %} {
    todo!()
  }
  {% endfor %}

  {% for meth in obj.methods() -%}
  {%- match meth.return_type() -%}

  {%- when Some with (return_type) -%}
  pub fn {{ meth.name()|fn_name_rs }}({% call rs::self_arg(meth) %}, {% call rs::arg_list_decl(meth.arguments()) %}) -> {% call rs::fallible_return_type(meth, return_type|type_rs) %} {
    todo!()
  }

  {%- when None -%}
  pub fn {{ meth.name()|fn_name_rs }}({% call rs::self_arg(meth) %}, {% call rs::arg_list_decl(meth.arguments()) %}) -> {% call rs::fallible_return_type(meth, "()") %} {
    todo!()
  }
  {% endmatch %}
  {% endfor %}
}
