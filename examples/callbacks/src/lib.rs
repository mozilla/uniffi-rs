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

include!(concat!(env!("OUT_DIR"), "/callbacks.uniffi.rs"));
