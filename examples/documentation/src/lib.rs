/// Person with a name.
pub struct Person {
    /// Person's name.
    pub name: String,
}

/// Create hello message to a `person`.
pub fn hello(person: Person) -> String {
    let name = person.name;
    format!("Hello {name}!")
}

/// Add two integers together.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

uniffi::include_scaffolding!("documentation");
