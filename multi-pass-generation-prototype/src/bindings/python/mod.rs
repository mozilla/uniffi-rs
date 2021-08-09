use crate::udl::UdlItem;
use crate::generate::VecPass;
use crate::log::FileLogger;
use crate::passes::add_nested_types;
use std::fs::File;
use std::io::Write;

fn log_input(input: &Vec<UdlItem>) {
    let content: String = input.iter().map(|i| format!("< {:?}\n", i)).collect();
    File::create("00-input.log").unwrap().write(content.as_bytes()).unwrap();
}

pub fn render_udl_to_file(input: Vec<UdlItem>, _file: File) {
    log_input(&input);
    input
        .run_pass(&add_nested_types(FileLogger::new("01-add-nested-types.log")));
}
