/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;

macro_rules! impl_code_type_for_miscellany {
    ($T:ident, $canonical_name:literal) => {
        #[derive(Debug)]
        pub struct $T;

        impl CodeType for $T {
            fn type_label(&self) -> String {
                format!("{}", $canonical_name)
            }

            fn canonical_name(&self) -> String {
                format!("{}", $canonical_name)
            }
        }
    };
}

impl_code_type_for_miscellany!(TimestampCodeType, "Timestamp");

impl_code_type_for_miscellany!(DurationCodeType, "Duration");
