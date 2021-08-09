// CodeBlock -- IR format for code generation
//
// A CodeBlock represents 1 or more contiguous lines in the generated code

use crate::udl::{UdlItem, TypeItem, RecordDef};
use serde::Serialize;
use handlebars::{Handlebars, RenderError};
use serde_json::Value;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "template_name", content = "data")]
pub enum CodeBlock {
    Common, // Common code we always include
    FFIConverterPrimitiveClass,
    FFIConverterOptionClass,
    FFIConverterPrimitive {
        size: i32,
        pack_fmt: &'static str,
    },
    FFIConverterString,
    FFIConverterOption(TypeItem),
    RecordClassDef(RecordDef),
    FFIConverterRecord(RecordDef),
}

impl CodeBlock {
    pub fn udl_item_into_code_blocks(item: UdlItem) -> Vec<CodeBlock> {
        match item {
            UdlItem::Type(TypeItem::I32) => vec![
                CodeBlock::FFIConverterPrimitiveClass,
                CodeBlock::FFIConverterPrimitive {size: 4, pack_fmt: ">i"},
            ],
            UdlItem::Type(TypeItem::String) => vec![
                CodeBlock::FFIConverterString,
            ],
            UdlItem::Type(TypeItem::Option(inner)) => vec![
                CodeBlock::FFIConverterOptionClass,
                CodeBlock::FFIConverterOption(inner.as_ref().clone()),
            ],
            UdlItem::Type(TypeItem::Record(record_def)) => vec![
                CodeBlock::RecordClassDef(record_def.clone()),
                CodeBlock::FFIConverterRecord(record_def.clone()),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateRenderer {
    pub template_name: String,
    pub data: Value,
}

impl TemplateRenderer {
    pub fn render(&self, handlebars: &Handlebars) -> Result<String, RenderError> {
        handlebars.render(self.template_name.as_ref(), &self.data)
    }
}

impl From<CodeBlock> for TemplateRenderer {
    fn from(code_block: CodeBlock) -> TemplateRenderer {
        // We're going to leverage serde_json to avoid a ton of boilerplate
        let mut json_value = serde_json::value::to_value(code_block).unwrap();
        TemplateRenderer {
            template_name: json_value["template_name"].take().as_str().unwrap().into(),
            data: json_value["data"].take(),
        }
    }
}

