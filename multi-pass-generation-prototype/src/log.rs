/// Log what happens durring a pass

use std::fmt::Debug;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::cell::RefCell;

pub trait Logger {
    fn write(&self, line: String);

    // Log that we inputted an item
    fn log_input<I: Debug>(&self, item: I) {
        self.write(format!("< {:?}\n", item));
    }

    // Log that we outputted an item
    fn log_output<I: Debug>(&self, item: I) {
        self.write(format!("> {:?}\n", item));
    }

    // Log that we would have outputted an item, but it was a duplicate
    fn log_dupe<I: Debug>(&self, item: I) {
        self.write(format!("X {:?}\n", item));
    }

    // log a separate line
    fn log_separator(&self) {
        self.write("\n".into());
    }
}


/// Log to a file
pub struct FileLogger(RefCell<File>);

impl FileLogger {
    pub fn new(path: impl AsRef<Path>) -> Self {
        FileLogger(RefCell::new(File::create(path).expect("Error opening log file")))
    }
}

impl Logger for FileLogger {
    fn write(&self, line: String) {
        self.0.borrow_mut().write(line.as_bytes()).expect("Error writing to log file");
    }
}

/// Log to an internal string (useful for unit tests)
pub struct StringLogger(RefCell<String>);

impl StringLogger {
    pub fn new() -> Self {
        StringLogger(RefCell::new(String::new()))
    }

    pub fn value(&self) -> String {
        self.0.borrow().clone()
    }
}

impl Logger for StringLogger {
    fn write(&self, line: String) {
        self.0.borrow_mut().push_str(&line);
    }
}

pub struct NullLogger;

impl Logger for NullLogger {
    fn write(&self, _line: String) { }
}
