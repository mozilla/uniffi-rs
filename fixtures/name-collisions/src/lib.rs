use std::sync::{Arc, Mutex};

// Simple struct that might collide with other fixtures
#[derive(uniffi::Object)]
pub struct Logger {
    messages: Arc<Mutex<Vec<String>>>,
}

#[uniffi::export]
impl Logger {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn log(&self, message: String) {
        self.messages.lock().unwrap().push(message);
    }

    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

// This trait will be exported with foreign implementations - needs Send + Sync
#[uniffi::export(with_foreign)]
pub trait Processor: Send + Sync {
    fn process(&self, data: String) -> String;
}

// A struct that implements the trait locally
#[derive(uniffi::Object)]
pub struct LocalProcessor {
    prefix: String,
}

#[uniffi::export]
impl LocalProcessor {
    #[uniffi::constructor]
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

impl Processor for LocalProcessor {
    fn process(&self, data: String) -> String {
        format!("{}: {}", self.prefix, data)
    }
}

// Global functions that might collide with callback fixture
#[uniffi::export]
pub fn log_message(message: String) -> String {
    format!("collision-fixture: {}", message)
}

#[uniffi::export]
pub fn process_data(data: String) -> String {
    format!("processed: {}", data)
}

// Functions that collide with callbacks fixture
#[uniffi::export]
pub fn get_string(v: String, arg2: bool) -> String {
    format!("name-collisions get_string: {} (arg2: {})", v, arg2)
}

#[uniffi::export]
pub fn get_bool(v: bool, argument_two: bool) -> bool {
    !v && argument_two // Different logic than callbacks fixture
}

// A trait that might collide with callback fixture interfaces
#[uniffi::export(with_foreign)]
pub trait ForeignGetters: Send + Sync {
    fn get_string(&self, v: String, arg2: bool) -> String;
    fn get_bool(&self, v: bool, argument_two: bool) -> bool;
}

// Local implementation of the collision trait
pub struct LocalForeignGetters;

impl ForeignGetters for LocalForeignGetters {
    fn get_string(&self, v: String, arg2: bool) -> String {
        format!("LOCAL ForeignGetters: {} ({})", v, arg2)
    }
    
    fn get_bool(&self, v: bool, argument_two: bool) -> bool {
        v || argument_two
    }
}

// Function that uses both local and callback fixture types to test collision
#[uniffi::export]
pub fn test_collision_scenario() -> String {
    let logger = Logger::new();
    logger.log("Testing collision".to_string());
    
    // Try to use callback fixture if available
    let messages = logger.get_messages();
    format!("Local collision test completed with {} messages", messages.len())
}

uniffi::include_scaffolding!("name_collisions");
