// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/uitests/src/typedef_extern.udl");

fn main() { /* empty main required by `trybuild` */}
