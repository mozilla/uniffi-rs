/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod from_ci;
mod nodes;
mod sort;

pub use nodes::*;

// Note: the interface code is currently in a bit of a transition state. Most languages are still
// using the `ComponentInterface` to render their bindings, but we plan to switch them over to this
// system.
//
// Once we do that, we can rework the metadata code to generate a `BindingsIr` directly,
// delete a ton of the older code, and consider folding the contents of the `ir` module into
// `interface`.
