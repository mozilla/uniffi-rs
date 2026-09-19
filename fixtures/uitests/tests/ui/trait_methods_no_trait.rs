fn main() { /* empty main required by `trybuild` */}

// Try exporting Display for a type that doesn't implement the trait
#[derive(uniffi::Object)]
#[uniffi::export(Display)]
pub struct TraitMethods {}

uniffi::setup_scaffolding!();
