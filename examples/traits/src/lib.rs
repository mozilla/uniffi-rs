/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

// namespace functions.
fn get_buttons() -> Vec<Arc<dyn Button>> {
    vec![Arc::new(StopButton {}), Arc::new(GoButton {})]
}

fn press(button: Arc<dyn Button>) -> Arc<dyn Button> {
    button
}

pub trait Button: Send + Sync {
    fn name(&self) -> String;
}

struct GoButton {}

impl Button for GoButton {
    fn name(&self) -> String {
        "go".to_string()
    }
}

struct StopButton {}

impl Button for StopButton {
    fn name(&self) -> String {
        "stop".to_string()
    }
}

include!(concat!(env!("OUT_DIR"), "/traits.uniffi.rs"));
