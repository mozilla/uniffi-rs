/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// This entire crate is just a set of tests for metadata handling.  We use a separate crate
/// for testing because the metadata handling is split up between several crates, and no crate
/// on all the functionality.
#[cfg(test)]
mod tests;

pub struct UniFfiTag;
