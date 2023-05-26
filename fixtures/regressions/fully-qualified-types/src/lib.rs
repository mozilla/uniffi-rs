/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The point of this test is that we don't have any of the used Rust types
// in scope under their usual names, so we don't import the ones that aren't
// in the prelude, and we overrwrite the ones that are.
//
// Real Rust code wouldn't do this of course, but it's nice to be robust
// against it.
//
// We could consider giving `String` and `Box` similar treatment in future,
// but our use of those as unqualified names is not a recent regression,
// so that's for follow-up work.

#[allow(dead_code)]
type Vec = ();

#[allow(dead_code)]
type Option = ();

mod t {
    pub use std::collections::HashMap;
    pub use std::option::Option;
    pub use std::time::{Duration, SystemTime};
    pub use std::vec::Vec;
}

pub struct Values {
    d: t::Duration,
    t: t::SystemTime,
}

pub fn test() -> t::Option<t::HashMap<String, t::Vec<Values>>> {
    None
}

uniffi::include_scaffolding!("test");
