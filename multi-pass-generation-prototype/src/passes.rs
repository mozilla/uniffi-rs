// Passes that can be shared between all the bindings

use crate::udl::UdlItem;
use crate::generate::UniqueMap;
use crate::log::Logger;

// Extract nested types from all UDL items.
pub fn add_nested_types<L: Logger>(logger: L) -> UniqueMap<UdlItem, UdlItem, L> {
    UniqueMap {
        map_func: Box::new(UdlItem::into_self_and_descendents),
        logger,
    }
}

