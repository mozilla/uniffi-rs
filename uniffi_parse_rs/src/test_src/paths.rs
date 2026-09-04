use url::Url;

#[derive(uniffi::Record)]
struct TestRecord { }

#[derive(uniffi::Record)]
struct r#break { }

uniffi::use_remote_type!(paths3::RemoteRecord);

// Use the custom_type! impl from paths3
// This is a bit weird because there's also a `use url::Url` import at the top.
// This is not a name conflict because `use_remote_type` just instructs UniFFI which crate
// implements the FfiConverter traits.
uniffi::use_remote_type!(paths3::Url);

mod mod1 {
    // Test a renaming via a use statement
    use mod2::Mod2Record as Mod2RecordRenamed;
    // Test a glob import via a use statement
    use mod2::mod3::*;
    // These use statements form a cycle that we should detect
    use mod2::CircularUseImport;

    #[derive(uniffi::Record)]
    struct Mod1Record { }

    mod mod2 {
        #[derive(uniffi::Record)]
        struct Mod2Record { }

        mod mod3 {
            #[derive(uniffi::Record)]
            struct Mod3Record { }

            use crate::mod1::CircularUseImport;

            #[derive(uniffi::Error)]
            enum Error { }

            pub type Result<T> = ::std::result::Result<T, E=Error>;
        }

        use mod3::CircularUseImport;
    }

    pub type TestRecordAlias = mod2::TestRecord;
}

mod mod4 {
    #[derive(uniffi::Record)]
    struct Mod4Record { }
}

// Function with the same name as a module.
//
// This is allowed in Rust, since functions and modules live in different namespaces.
// We need to test that this doesn't trip up the path resolution.
#[uniffi::export]
pub fn mod4() {}

// Importing an item from an unknown crate with the same name
//
// Again, this would be allowed in Rust since macros live in a 3rd namespace, but we want to test
// that it doesn't mess up path resolution.
use ::macros::mod4;

// module who's name matches another crate
mod paths2 {
    #[derive(uniffi::Record)]
    struct AmbiguousRecord { }
}

use mod1::mod2::mod3::{self};

// Import the same item several different ways
use mod1::mod2::Mod2Record;
use crate::mod1::mod2::Mod2Record;
use mod1::Mod2RecordRenamed as Mod2Record;

mod mod5 {
    // Import the same item several different ways using glob imports
    use super::*;
    use crate::mod1::mod2::*;
}

// `self::` paths: `self` refers to the current module.
// The `self::` prefix is how real-world code disambiguates a module from an
// external crate with the same name (e.g. a `http` module vs the `http` crate).
mod mod6 {
    mod inner {
        #[derive(uniffi::Record)]
        pub struct SelfGlobRecord { }

        #[derive(uniffi::Record)]
        pub struct SelfUseRecord { }
    }

    // Glob re-export through `self::`
    pub use self::inner::*;
    // Named import through `self::`
    use self::inner::SelfUseRecord as SelfUseRecordRenamed;
}

// `use_remote_type!` whose path doesn't resolve: the macro's semantics are "the type
// named by the LAST segment, in the invoking module's scope" — the first segment only
// names the crate implementing the FFI traits.  Resolution must fall through to the
// module's regular items instead of failing.
mod remote_fallthrough {
    pub type ExternalRemote = unparsed_crate::ExternalRemote;

    uniffi::use_remote_type!(paths3::ExternalRemote);
}
