// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/uitests/src/trait_methods_unknown_trait.udl");

fn main() { /* empty main required by `trybuild` */}

// We derive all required traits, plus one we don't know about.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TraitMethods {}

impl std::fmt::Display for TraitMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TraitMethods()")
    }
}
