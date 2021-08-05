// Record type {{ rec.name() }}
#[derive(Clone, Eq, PartialEq)]
pub struct {{ rec.name()|class_name_rs }} {
    {%- call rs::arg_list_decl(rec.fields()) -%}
}

impl {{ rec.name()|class_name_rs }} {
    pub fn new({%- call rs::arg_list_decl(rec.fields()) -%}) -> Self {
        Self {
            {%- for field in rec.fields() %}
            {{ field.name()|var_name_rs }},
            {%- endfor %}
        }
    }
}
