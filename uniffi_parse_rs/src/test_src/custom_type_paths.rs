// Custom types resolved by the type they cover, not the module the macro was invoked in.
//
// The `custom_type!` invocations live in `registrations`; other modules name the
// underlying types directly (through aliases, non-UniFFI structs, or paths into
// unparsed crates) and must still resolve to the custom type.

mod types {
    // A non-UniFFI type covered by a custom type registered in another module
    pub struct Concrete { }

    // An alias to a type from an unparsed crate, covered by a custom type in another module
    pub type ExternalAlias = external_crate::ExternalType;
}

mod registrations {
    use crate::types::{Concrete, ExternalAlias};

    uniffi::custom_type!(Concrete, String, {
        remote,
        into: |obj| obj.to_string(),
        try_from: |s| s.parse(),
    });

    uniffi::custom_type!(ExternalAlias, String, {
        remote,
        into: |obj| obj.to_string(),
        try_from: |s| s.parse(),
    });

    // A custom type over a local alias to an unparsed crate.  Other modules import the
    // external type directly (`use external_crate2::Direct`) and never see this alias.
    type Direct = external_crate2::Direct;

    uniffi::custom_type!(Direct, String, {
        remote,
        into: |obj| obj.to_string(),
        try_from: |s| s.parse(),
    });
}

mod usage {
    use external_crate2::Direct;

    use crate::types::{Concrete, ExternalAlias};

    #[derive(uniffi::Record)]
    pub struct UsesCustomTypes {
        pub concrete: Concrete,
        pub aliased: ExternalAlias,
        pub direct: Direct,
    }
}
