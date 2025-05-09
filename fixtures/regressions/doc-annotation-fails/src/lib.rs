/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = std::concat!("A", "has a", "B")]
#[derive(uniffi::Enum)]
pub enum ConcattedDocEnum {
    B,
}

#[doc = "A has a B"]
#[derive(uniffi::Enum)]
pub enum PlainDocAnnotation {
    B,
}

uniffi::include_scaffolding!("test");
