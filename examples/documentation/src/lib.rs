use std::sync::RwLock;

/// Pet with a name.
pub struct Pet {
    /// Pet's name.
    pub name: String,
}

/// Create hello message to a `pet`.
pub fn hello(pet: Pet) -> String {
    let name = pet.name;
    format!("Hello {name}!")
}

/// Person with a name.
pub struct Person {
    name: RwLock<String>,
}

impl Person {
    /// Create new person with [name].
    pub fn new(name: String) -> Self {
        Person {
            name: RwLock::new(name),
        }
    }

    /// Set person name.
    pub fn set_name(&self, name: String) {
        *self.name.write().unwrap() = name;
    }

    /// Get person's name.
    pub fn get_name(&self) -> String {
        self.name.read().unwrap().clone()
    }
}

/// Add two integers together.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Test enum.
pub enum TestEnum {
    /// Variant A.
    A,

    /// Variant B.
    B,

    /// Variant C.
    C,
}

uniffi::include_scaffolding!("documentation");
