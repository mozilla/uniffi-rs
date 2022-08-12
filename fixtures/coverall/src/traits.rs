/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

// namespace functions.
pub fn get_traits() -> Vec<Arc<dyn TestTrait>> {
    vec![
        Arc::new(Trait1 {
            ..Default::default()
        }),
        Arc::new(Trait2 {}),
    ]
}

pub trait TestTrait: Send + Sync + std::fmt::Debug {
    fn name(&self) -> String;

    fn number(self: Arc<Self>) -> u64;

    fn strong_count(self: Arc<Self>) -> u64 {
        Arc::strong_count(&self) as u64
    }

    fn take_other(&self, other: Option<Arc<dyn TestTrait>>);

    fn get_other(&self) -> Option<Arc<dyn TestTrait>>;
}

#[derive(Debug, Default)]
pub(crate) struct Trait1 {
    // A reference to another trait.
    other: Mutex<Option<Arc<dyn TestTrait>>>,
}

impl TestTrait for Trait1 {
    fn name(&self) -> String {
        "trait 1".to_string()
    }

    fn number(self: Arc<Self>) -> u64 {
        1_u64
    }

    fn take_other(&self, other: Option<Arc<dyn TestTrait>>) {
        *self.other.lock().unwrap() = other.map(|arc| Arc::clone(&arc))
    }

    fn get_other(&self) -> Option<Arc<dyn TestTrait>> {
        (*self.other.lock().unwrap()).as_ref().map(Arc::clone)
    }
}

#[derive(Debug)]
pub(crate) struct Trait2 {}
impl TestTrait for Trait2 {
    fn name(&self) -> String {
        "trait 2".to_string()
    }

    fn number(self: Arc<Self>) -> u64 {
        2_u64
    }

    // Don't bother implementing these here - the test on the struct above is ok.
    fn take_other(&self, _other: Option<Arc<dyn TestTrait>>) {
        unimplemented!();
    }

    fn get_other(&self) -> Option<Arc<dyn TestTrait>> {
        unimplemented!()
    }
}
