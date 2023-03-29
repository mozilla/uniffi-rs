/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(env!("OUT_DIR"), "/docstring.uniffi.rs"));

enum EnumTest {
    One,
}

enum AssociatedEnumTest {
    Test{},
}

#[derive(Debug, thiserror::Error)]
enum ErrorTest {
    #[error("Test")]
    One,
}

#[derive(Debug, thiserror::Error)]
enum AssociatedErrorTest {
    #[error("Test")]
    Test{},
}

struct ObjectTest {

}

impl ObjectTest {
    pub fn new() -> Self {
        ObjectTest{}
    }

    pub fn new_alternate() -> Self {
        ObjectTest{}
    }

    pub fn test(&self) {
    }
}

struct RecordTest {
    test: i32,
}

pub fn test() {
    let _ = ErrorTest::One;
}

pub trait CallbackTest {
   fn test(&self); 
}
