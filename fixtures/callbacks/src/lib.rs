/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

trait ForeignGetters {
    fn get_bool(&self, v: bool, arg2: bool) -> bool;
    fn get_string(&self, v: String, arg2: bool) -> String;
    fn get_option(&self, v: Option<String>, arg2: bool) -> Option<String>;
    fn get_list(&self, v: Vec<i32>, arg2: bool) -> Vec<i32>;
}

#[derive(Debug, Clone)]
pub struct RustGetters;

impl RustGetters {
    pub fn new() -> Self {
        RustGetters
    }
    fn get_bool(&self, callback: Box<dyn ForeignGetters>, v: bool, arg2: bool) -> bool {
        callback.get_bool(v, arg2)
    }
    fn get_string(&self, callback: Box<dyn ForeignGetters>, v: String, arg2: bool) -> String {
        callback.get_string(v, arg2)
    }
    fn get_option(
        &self,
        callback: Box<dyn ForeignGetters>,
        v: Option<String>,
        arg2: bool,
    ) -> Option<String> {
        callback.get_option(v, arg2)
    }
    fn get_list(&self, callback: Box<dyn ForeignGetters>, v: Vec<i32>, arg2: bool) -> Vec<i32> {
        callback.get_list(v, arg2)
    }

    fn get_string_optional_callback<'a>(&self, callback: Option<Box<dyn ForeignGetters + 'a>>, v: String, arg2: bool) -> Option<String> {
        callback.map(|c| c.get_string(v, arg2))
    }
}

impl Default for RustGetters {
    fn default() -> Self {
        Self::new()
    }
}

// Use `Send+Send` because we want to store the callback in an exposed
// `Send+Sync` object.
trait StoredForeignStringifier: Send + Sync + std::fmt::Debug {
    fn from_simple_type(&self, value: i32) -> String;
    fn from_complex_type(&self, values: Option<Vec<Option<f64>>>) -> String;
}

#[derive(Debug)]
pub struct RustStringifier {
    callback: Box<dyn StoredForeignStringifier>,
}

impl RustStringifier {
    fn new(callback: Box<dyn StoredForeignStringifier>) -> Self {
        RustStringifier { callback }
    }

    fn from_simple_type(&self, value: i32) -> String {
        self.callback.from_simple_type(value)
    }
}

include!(concat!(env!("OUT_DIR"), "/callbacks.uniffi.rs"));
