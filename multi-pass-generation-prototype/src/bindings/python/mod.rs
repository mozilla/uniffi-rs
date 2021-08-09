mod codeblocks;
mod passes;

use codeblocks::CodeBlock;

use crate::udl::UdlItem;
use crate::generate::VecPass;
use crate::log::FileLogger;
use crate::passes::add_nested_types;
use passes::*;
use std::fs::File;
use std::io::Write;

fn log_input(input: &Vec<UdlItem>) {
    let content: String = input.iter().map(|i| format!("< {:?}\n", i)).collect();
    File::create("00-input.log").unwrap().write(content.as_bytes()).unwrap();
}

pub fn render_udl_to_file(input: Vec<UdlItem>, _file: File) {
    log_input(&input);
    input
        .run_pass(&add_nested_types(FileLogger::new("01-add-nested-types.log")))
        .run_pass(&map_udl_items_to_code_blocks(FileLogger::new("02-map-items-to-code-blocks.log")))
        .run_pass(&add_common_code_and_sort_codeblocks(FileLogger::new("03-add-common-code-and-sort.log")));
}
