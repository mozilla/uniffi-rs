/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
pub struct TodoEntry {
    text: String,
}

#[derive(Debug, thiserror::Error)]
enum TodoError {
    #[error("The todo does not exist!")]
    TodoDoesNotExist,
    #[error("The todolist is empty!")]
    EmptyTodoList,
    #[error("That todo already exists!")]
    DuplicateTodo,
    #[error("Empty String error!")]
    EmptyString,
}

fn create_entry_with<S: Into<String>>(item: S) -> Result<TodoEntry> {
    let text = item.into();
    if text == "" {
        return Err(TodoError::EmptyString);
    }
    Ok(TodoEntry { text })
}

type Result<T, E = TodoError> = std::result::Result<T, E>;

// I am a simple Todolist
#[derive(Debug, Clone)]
pub struct TodoList {
    items: Vec<String>,
}

impl TodoList {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add_item<S: Into<String>>(&mut self, item: S) -> Result<()> {
        let item = item.into();
        if item == "" {
            return Err(TodoError::EmptyString);
        }
        if self.items.contains(&item) {
            return Err(TodoError::DuplicateTodo);
        }
        self.items.push(item.into());
        Ok(())
    }

    fn get_last(&self) -> Result<String> {
        self.items
            .last()
            .cloned()
            .ok_or_else(|| TodoError::EmptyTodoList)
    }

    fn get_first(&self) -> Result<String> {
        self.items
            .first()
            .cloned()
            .ok_or_else(|| TodoError::EmptyTodoList)
    }

    fn add_entries(&mut self, entries: Vec<TodoEntry>) {
        self.items.extend(entries.into_iter().map(|e| e.text))
    }

    fn add_entry(&mut self, entry: TodoEntry) -> Result<()> {
        self.add_item(entry.text)
    }

    fn add_items<S: Into<String>>(&mut self, items: Vec<S>) {
        self.items.extend(items.into_iter().map(Into::into))
    }

    fn get_items(&self) -> Vec<String> {
        self.items.clone()
    }

    fn get_entries(&self) -> Vec<TodoEntry> {
        self.items
            .iter()
            .map(|text| TodoEntry { text: text.clone() })
            .collect()
    }

    fn get_last_entry(&self) -> Result<TodoEntry> {
        let text = self.get_last()?;
        Ok(TodoEntry { text })
    }

    fn clear_item<S: Into<String>>(&mut self, item: S) -> Result<()> {
        let item = item.into();
        let idx = self
            .items
            .iter()
            .position(|s| s == &item)
            .ok_or_else(|| TodoError::TodoDoesNotExist)?;
        self.items.remove(idx);
        Ok(())
    }
}

include!(concat!(env!("OUT_DIR"), "/todolist.uniffi.rs"));
