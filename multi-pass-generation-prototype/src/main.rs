use std::fs::File;
use multi_pass_generation_prototype::bindings::python::render_udl_to_file;
use multi_pass_generation_prototype::udl::{ TypeItem, RecordDef };

fn main() {
    render_udl_to_file(vec![
        TypeItem::I32.into(),
        TypeItem::Record(RecordDef {
            name: "Foo".into(),
            fields: vec![
                ('a'.into(), TypeItem::I32),
                ('b'.into(), TypeItem::String),
            ],
        }).into(),
    ], File::create("output.py").unwrap());
}
