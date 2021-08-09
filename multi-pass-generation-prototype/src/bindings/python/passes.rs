// Python-specific passes

use super::{CodeBlock, TemplateRenderer};
use crate::generate::{ Map, UniqueMap, Transform };
use crate::log::Logger;
use crate::udl::UdlItem;

pub fn map_udl_items_to_code_blocks<L: Logger>(logger: L) -> UniqueMap<UdlItem, CodeBlock, L> {
    UniqueMap {
        map_func: Box::new(CodeBlock::udl_item_into_code_blocks),
        logger,
    }
}

pub fn add_common_code_and_sort_codeblocks<L: Logger>(logger: L) -> Transform<CodeBlock, CodeBlock, L> {
    Transform {
        map_func: Box::new(|mut input| {
            input.push(CodeBlock::Common);
            input.sort();
            input
        }),
        logger,
    }
}

pub fn map_codeblocks_to_template_renderer<L: Logger>(logger: L) -> Map<CodeBlock, TemplateRenderer, L> {
    Map {
        map_func: Box::new(|codeblock| vec![codeblock.into()]),
        logger,
    }
}
