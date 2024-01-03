// This crate does not use UniFFI, but it exposes some types which are used by crates which do.
// This type is referenced as an "external" type as a dictionary.
pub struct ExternalCrateDictionary {
    pub sval: String,
}

pub struct ExternalCrateInterface {
    pub sval: String,
}

#[non_exhaustive]
pub enum ExternalCrateNonExhaustiveEnum {
    One,
    Two,
}

// This type is referenced as an "external" type as an interface.
impl ExternalCrateInterface {
    pub fn new(sval: String) -> Self {
        ExternalCrateInterface { sval }
    }

    pub fn value(&self) -> String {
        self.sval.clone()
    }
}
