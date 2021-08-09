// Python-specific passes

use super::CodeBlock;
use crate::generate::Map;
use crate::log::Logger;
use crate::udl::UdlItem;

// Map UDL items -> code blocks
pub fn map_udl_items_to_code_blocks<L: Logger>(logger: L) -> Map<UdlItem, CodeBlock, L> {
    Map {
        map_func: Box::new(CodeBlock::udl_item_into_code_blocks),
        logger,
        prevent_dupes: true
    }
}

