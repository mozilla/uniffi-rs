/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{ComplexError, CoverallError};
use std::sync::{Arc, Mutex};

// namespace functions.
pub fn get_traits() -> Vec<Arc<dyn NodeTrait>> {
    vec![Arc::new(Trait1::default()), Arc::new(Trait2::default())]
}

#[uniffi::trait_interface]
pub trait NodeTrait: Send + Sync + std::fmt::Debug {
    fn name(&self) -> String;

    fn set_parent(&self, parent: Option<Arc<dyn NodeTrait>>);

    fn get_parent(&self) -> Option<Arc<dyn NodeTrait>>;

    fn strong_count(self: Arc<Self>) -> u64 {
        Arc::strong_count(&self) as u64
    }
}

pub fn ancestor_names(node: Arc<dyn NodeTrait>) -> Vec<String> {
    let mut names = vec![];
    let mut parent = node.get_parent();
    while let Some(node) = parent {
        names.push(node.name());
        parent = node.get_parent();
    }
    names
}

/// Test trait
///
/// The goal here is to test all possible arg, return, and error types.
#[uniffi::trait_interface]
pub trait Getters: Send + Sync {
    fn get_bool(&self, v: bool, arg2: bool) -> bool;
    fn get_string(&self, v: String, arg2: bool) -> Result<String, CoverallError>;
    fn get_option(&self, v: String, arg2: bool) -> Result<Option<String>, ComplexError>;
    fn get_list(&self, v: Vec<i32>, arg2: bool) -> Vec<i32>;
    fn get_nothing(&self, v: String);
}

struct RustGetters;

impl Getters for RustGetters {
    fn get_bool(&self, v: bool, arg2: bool) -> bool {
        v ^ arg2
    }

    fn get_string(&self, v: String, arg2: bool) -> Result<String, CoverallError> {
        if v == "too-many-holes" {
            Err(CoverallError::TooManyHoles)
        } else if v == "unexpected-error" {
            panic!("unexpected error")
        } else if arg2 {
            Ok(v.to_uppercase())
        } else {
            Ok(v)
        }
    }

    fn get_option(&self, v: String, arg2: bool) -> Result<Option<String>, ComplexError> {
        if v == "os-error" {
            Err(ComplexError::OsError {
                code: 100,
                extended_code: 200,
            })
        } else if v == "unknown-error" {
            Err(ComplexError::UnknownError)
        } else if arg2 {
            if !v.is_empty() {
                Ok(Some(v.to_uppercase()))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(v))
        }
    }

    fn get_list(&self, v: Vec<i32>, arg2: bool) -> Vec<i32> {
        if arg2 {
            v
        } else {
            vec![]
        }
    }

    fn get_nothing(&self, _v: String) {}
}

pub fn make_rust_getters() -> Arc<dyn Getters> {
    Arc::new(RustGetters)
}

pub fn test_getters(getters: Arc<dyn Getters>) {
    assert!(!getters.get_bool(true, true));
    assert!(getters.get_bool(true, false));
    assert!(getters.get_bool(false, true));
    assert!(!getters.get_bool(false, false));

    assert_eq!(
        getters.get_string("hello".to_owned(), false).unwrap(),
        "hello"
    );
    assert_eq!(
        getters.get_string("hello".to_owned(), true).unwrap(),
        "HELLO"
    );

    assert_eq!(
        getters.get_option("hello".to_owned(), true).unwrap(),
        Some("HELLO".to_owned())
    );
    assert_eq!(
        getters.get_option("hello".to_owned(), false).unwrap(),
        Some("hello".to_owned())
    );
    assert_eq!(getters.get_option("".to_owned(), true).unwrap(), None);

    assert_eq!(getters.get_list(vec![1, 2, 3], true), vec![1, 2, 3]);
    assert_eq!(getters.get_list(vec![1, 2, 3], false), Vec::<i32>::new());

    // Call get_nothing to make sure it doesn't panic.  There's no point in checking the output
    // though
    getters.get_nothing("hello".to_owned());

    assert_eq!(
        getters.get_string("too-many-holes".to_owned(), true),
        Err(CoverallError::TooManyHoles)
    );
    assert_eq!(
        getters.get_option("os-error".to_owned(), true),
        Err(ComplexError::OsError {
            code: 100,
            extended_code: 200
        })
    );
    assert_eq!(
        getters.get_option("unknown-error".to_owned(), true),
        Err(ComplexError::UnknownError)
    );
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        getters.get_string("unexpected-error".to_owned(), true)
    }));
    assert!(result.is_err());
}

#[derive(Debug, Default)]
pub(crate) struct Trait1 {
    // A reference to another trait.
    parent: Mutex<Option<Arc<dyn NodeTrait>>>,
}

impl NodeTrait for Trait1 {
    fn name(&self) -> String {
        "node-1".to_string()
    }

    fn set_parent(&self, parent: Option<Arc<dyn NodeTrait>>) {
        *self.parent.lock().unwrap() = parent.map(|arc| Arc::clone(&arc))
    }

    fn get_parent(&self) -> Option<Arc<dyn NodeTrait>> {
        (*self.parent.lock().unwrap()).as_ref().map(Arc::clone)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Trait2 {
    parent: Mutex<Option<Arc<dyn NodeTrait>>>,
}
impl NodeTrait for Trait2 {
    fn name(&self) -> String {
        "node-2".to_string()
    }

    fn set_parent(&self, parent: Option<Arc<dyn NodeTrait>>) {
        *self.parent.lock().unwrap() = parent.map(|arc| Arc::clone(&arc))
    }

    fn get_parent(&self) -> Option<Arc<dyn NodeTrait>> {
        (*self.parent.lock().unwrap()).as_ref().map(Arc::clone)
    }
}
