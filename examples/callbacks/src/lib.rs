/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub trait OnCallAnswered {
    fn hello(&self) -> String;
    fn busy(&self);
    fn text_received(&self, text: String);
}

#[derive(Debug, Clone)]
struct Telephone;
impl Telephone {
    fn new() -> Self {
        Telephone
    }
    fn call(&self, domestic: bool, call_responder: Box<dyn OnCallAnswered>) {
        if domestic {
            let _ = call_responder.hello();
        } else {
            call_responder.busy();
            call_responder.text_received("Not now, I'm on another call!".into());
        }
    }
}

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
}

impl Default for RustGetters {
    fn default() -> Self {
        Self::new()
    }
}

// Use Send if we want to store the callback in an exposed object.
trait StoredForeignStringifier: Send + std::fmt::Debug {
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
