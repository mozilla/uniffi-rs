/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
struct TodoEntry {
    text: String,
}

// I am a simple Todolist
#[derive(Debug, Clone)]
struct TodoList {
    items: Vec<String>,
}

impl TodoList {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add_item<S: Into<String>>(&mut self, item: S) {
        self.items.push(item.into())
    }

    fn get_last(&self) -> String {
        self.items.last().cloned().unwrap()
    }

    fn add_entry(&mut self, entry: TodoEntry) {
        self.items.push(entry.text)
    }
}

include!(concat!(env!("OUT_DIR"), "/todolist.uniffi.rs"));
