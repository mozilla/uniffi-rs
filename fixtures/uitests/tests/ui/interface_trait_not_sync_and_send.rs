// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/uitests/src/trait.udl");

fn main() { /* empty main required by `trybuild` */}

// This will fail to compile, because the trait is not explicit Send+Sync
pub trait Trait {
}
