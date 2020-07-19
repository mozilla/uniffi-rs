/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


 // I am a simple Todolist
#[derive(Debug, Clone)]
struct TodoList {
    items: Vec<String>
}

impl TodoList {
    fn new() -> Self {
        Self {
            items: Vec::new()
        }
    }

    // It would be nice if we could also accept a String
    // as an argument, especially since we end up .to_string()
    // anyways, not a big deal, but definitely a limitation at the time...
    fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string())
    }

    fn get_last(&self) -> String {
        self.items.last().cloned().unwrap() // Well Should probably return the option, but I'm still figure it out :)
    }
}

include!(concat!(env!("OUT_DIR"), "/todolist.uniffi.rs"));
