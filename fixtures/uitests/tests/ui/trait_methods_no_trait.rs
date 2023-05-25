// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/trait-methods/src/trait_methods.udl");

fn main() { /* empty main required by `trybuild` */}

// We derive most required traits, just not `Display`, to keep the output smaller.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitMethods {}

impl TraitMethods {
    fn new(name: String) -> Self {
        unreachable!();
    }
}
