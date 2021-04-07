/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref NUM_ALIVE: RwLock<u64> = {
        RwLock::new(0)
    };
}

#[derive(Debug, thiserror::Error)]
enum CoverallError {
    #[error("The coverall has too many holes")]
    TooManyHoles,
}

#[derive(Debug, Clone)]
pub struct SimpleDict {
    text: String,
    maybe_text: Option<String>,
    a_bool: bool,
    maybe_a_bool: Option<bool>,
    unsigned8: u8,
    maybe_unsigned8: Option<u8>,
    unsigned64: u64,
    maybe_unsigned64: Option<u64>,
    signed8: i8,
    maybe_signed8: Option<i8>,
    signed64: i64,
    maybe_signed64: Option<i64>,
    float32: f32,
    maybe_float32: Option<f32>,
    float64: f64,
    maybe_float64: Option<f64>,
}

fn create_some_dict() -> SimpleDict {
    SimpleDict {
        text: "text".to_string(),
        maybe_text: Some("maybe_text".to_string()),
        a_bool: true,
        maybe_a_bool: Some(false),
        unsigned8: 1,
        maybe_unsigned8: Some(2),
        unsigned64: u64::MAX,
        maybe_unsigned64: Some(u64::MIN),
        signed8: 8,
        maybe_signed8: Some(0),
        signed64: i64::MAX,
        maybe_signed64: Some(0),
        float32: 1.2345,
        maybe_float32: Some(22.0 / 7.0),
        float64: 0.0,
        maybe_float64: Some(1.0),
    }
}

fn create_none_dict() -> SimpleDict {
    SimpleDict {
        text: "text".to_string(),
        maybe_text: None,
        a_bool: true,
        maybe_a_bool: None,
        unsigned8: 1,
        maybe_unsigned8: None,
        unsigned64: u64::MAX,
        maybe_unsigned64: None,
        signed8: 8,
        maybe_signed8: None,
        signed64: i64::MAX,
        maybe_signed64: None,
        float32: 1.2345,
        maybe_float32: None,
        float64: 0.0,
        maybe_float64: None,
    }
}

fn get_num_alive() -> u64 {
    *NUM_ALIVE.read().unwrap()
}

type Result<T, E = CoverallError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Coveralls {
    name: String,
}

impl Coveralls {
    fn new(name: String) -> Self {
        *NUM_ALIVE.write().unwrap() += 1;
        Self { name }
    }

    fn fallible_new(name: String, should_fail: bool) -> Result<Self> {
        if should_fail {
            Err(CoverallError::TooManyHoles)
        } else {
            Ok(Self::new(name))
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn panicing_new(message: String) -> Self {
        panic!("{}", message);
    }

    fn maybe_throw(&self, should_throw: bool) -> Result<bool> {
        if should_throw {
            Err(CoverallError::TooManyHoles)
        } else {
            Ok(true)
        }
    }

    fn panic(&self, message: String) {
        panic!("{}", message);
    }
}

impl Drop for Coveralls {
    fn drop(&mut self) {
        *NUM_ALIVE.write().unwrap() -= 1;
    }
}
include!(concat!(env!("OUT_DIR"), "/coverall.uniffi.rs"));
