// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/trait-methods/src/trait_methods.udl");

fn main() { /* empty main required by `trybuild` */}

// We derive most required traits, just not `Display`, to keep the output smaller.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitMethods {}

impl TraitMethods {
    fn new(_name: String) -> Self {
        unreachable!();
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UdlEnum {
    S { s: String },
    I { i: i8 },
}
