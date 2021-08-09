// CodeBlock -- IR format for code generation
//
// A CodeBlock represents 1 or more contiguous lines in the generated code

use crate::udl::{UdlItem, TypeItem, RecordDef};

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
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
