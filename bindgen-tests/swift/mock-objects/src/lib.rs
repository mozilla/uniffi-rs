/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Object, Debug)]
pub struct TestObject;

#[uniffi::export]
impl TestObject {
    pub fn is_mock(&self) -> bool {
        false
    }
}

uniffi::setup_scaffolding!("swift_mock_objects");
