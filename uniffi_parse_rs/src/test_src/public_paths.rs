
#[derive(uniffi::Record)]
pub struct Rec1 { }

pub mod mod1 {
    pub mod mod2 {
        #[derive(uniffi::Record)]
        pub struct Rec2 { }
    }

    // This is the public path to get to Rec3
    pub use crate::nonpub::Rec3;

    pub struct CustomType { }
}

mod nonpub {
    use super::mod1::CustomType;

    #[derive(uniffi::Record)]
    pub struct Rec3 { }

    #[derive(uniffi::Record)]
    pub struct Rec4 { }

    uniffi::custom_type!(CustomType, u64);
}

// Non-pub use, this should be ignored
use nonpub::Rec3;

pub mod mod3 {
    pub mod mod4 {
        // This is the public path to get to Rec, but it's longer than `mod1::Rec3` so it shouldn't
        // be used.
        pub use crate::nonpub::Rec3;
    }
}

pub use nonpub::Rec4 as RenamedRec4;


mod nonpub2 {
    #[derive(uniffi::Record)]
    pub struct Rec5 { }
}

pub mod mod5 {
    // Use glob that should pick up Rec5
    pub use super::nonpub2::*;
}

// Non-pub use glob, this should be ignored
use nonpub2::*;
