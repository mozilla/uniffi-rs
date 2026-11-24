cfg_select! {
    feature = "feature-a" => {
        #[derive(uniffi::Record)]
        pub struct RecordA;
    }
    feature = "feature-b" => {
        #[derive(uniffi::Record)]
        pub struct RecordB;
    }
    _ => {
        #[derive(uniffi::Record)]
        pub struct RecordC;
    }
}

cfg_select! {
    feature = "enabled-feature" => {
        // Nested cfg_select, to try to confuse the parser
        cfg_select! {
            _ => {
                // This module will only be available after both `cfg_select!` items are resolved.
                mod mod1 {
                    pub use std::cfg_select as cfg_select_renamed;

                    #[derive(uniffi::Record)]
                    pub struct RecordA;
                }
            }
        }
    }
    _ => {}
}

use mod1::*;

// Can't resolve this macro until the previous cfg_select! block is resolved
cfg_select_renamed! {
    _ => {
        mod mod2 {
            #[derive(uniffi::Record)]
            pub struct RecordB;

            pub type RecordAAlias = crate::mod1::RecordA;
        }
    }
}
