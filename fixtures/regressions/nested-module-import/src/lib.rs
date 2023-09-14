/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The generated scaffolding is imported into a submodule on the Rust side (for namespacing)
// and then reexported using `pub use *`.
// We hit a case where some required types weren't `pub` and thus this would fail.

pub struct Integer(i32);

pub fn run() -> Integer {
    Integer(42)
}

mod ffi {
    use super::*;
    uniffi::include_scaffolding!("test");

    impl UniffiCustomTypeConverter for Integer {
        type Builtin = i32;

        fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
            Ok(Integer(val))
        }

        fn from_custom(obj: Self) -> Self::Builtin {
            obj.0
        }
    }
}
pub use ffi::*;
