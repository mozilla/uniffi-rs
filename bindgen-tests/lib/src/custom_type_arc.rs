/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Regression test for a custom type pattern that works with 0.32.0, but no
// longer works as of d49730506ae93ac0b89ff6cc98f8bdfe15c14305.

use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct Foo(u32);

pub struct Bar(u32);

uniffi::custom_type!(Bar, Arc<Foo>, {
    try_lift: |val| Ok(Bar(val.0)),
    lower: |custom| Arc::new(Foo(custom.0)),
});
