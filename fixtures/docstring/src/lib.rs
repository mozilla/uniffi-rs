/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

include!(concat!(env!("OUT_DIR"), "/docstring.uniffi.rs"));

pub enum EnumTest {
    One,
    Two,
}

pub enum AssociatedEnumTest {
    Test{ code: i16 },
    Test2{ code: i16 },
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorTest {
    #[error("Test")]
    One,
    #[error("Two")]
    Two,
}

#[derive(Debug, thiserror::Error)]
pub enum AssociatedErrorTest {
    #[error("Test")]
    Test{ code: i16 },
    #[error("Test2")]
    Test2{ code: i16 },
}

pub struct ObjectTest {

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

pub struct RecordTest {
    test: i32,
}

pub fn test() {
    let _ = ErrorTest::One;
    let _ = ErrorTest::Two;
}

pub trait CallbackTest {
   fn test(&self); 
}
