// Python-specific passes

use super::CodeBlock;
use crate::generate::{ Map, Transform };
use crate::log::Logger;
use crate::udl::UdlItem;

pub fn map_udl_items_to_code_blocks<L: Logger>(logger: L) -> Map<UdlItem, CodeBlock, L> {
    Map {
        map_func: Box::new(CodeBlock::udl_item_into_code_blocks),
        logger,
        prevent_dupes: true
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
