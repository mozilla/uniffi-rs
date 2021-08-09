// Python-specific passes

use serde_json::{json, Value};
use tera::{Tera, Context};
use super::{CodeBlock, RenderInfo};
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

pub fn map_codeblocks_to_template_renderer<L: Logger>(logger: L) -> Map<CodeBlock, RenderInfo, L> {
    Map {
        map_func: Box::new(|codeblock| vec![codeblock.into()]),
        logger,
    }
}

pub fn render_templates<L: Logger>(logger: L) -> Map<RenderInfo, String, L> {
    let tera = Tera::new("templates/python/**/*.py").unwrap();
    Map {
        map_func: Box::new(move |template_renderer| {
            vec![tera.render(
                format!("{}.py", template_renderer.template_name).as_ref(),
                &Context::from_serialize(&(if let Value::Null = template_renderer.data { json!{ {} } } else {template_renderer.data})).unwrap()).unwrap().clone()
            ]
        }),
        logger,
    }
}
