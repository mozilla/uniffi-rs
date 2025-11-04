# Renaming via proc-macros

UniFFI allows you to rename all callables and user-defined types in the foreign bindings using the `name` attribute.

Renaming via proc-macros impacts all language bindings.
For language-specific renaming, which offers renaming enum varients, record members, args, etc, see [TOML-based renaming](../renaming.md).

## Examples

### Functions:

```rust
#[uniffi::export(name = "renamed_function")]
fn function(record: Record) -> Enum {
    Enum::Record(record)
}
```

### Records and Enums

```rust
#[derive(uniffi::Record)]
#[uniffi(name = "RenamedRecord")]
pub struct Record {
    item: i32,
}

#[derive(uniffi::Enum)]
#[uniffi(name = "RenamedEnum")]
pub enum Enum {
    VariantA,
    Record(Record),
}
```

### Objects, Traits, and methods

If you are renaming both the object and a callable, you must specify the new name in both the `derive` and the `uniffi::export` macros.

Traits cannot yet be renamed, but trait methods can be renamed as usual.

```rust
#[derive(uniffi::Object)]
#[uniffi(name = "RenamedObject")]
pub struct Object {
    value: i32,
}

#[uniffi::export(name = "RenamedObject")]
impl Object {
    #[uniffi::constructor(name = "renamed_constructor")]
    pub fn new(value: i32) -> Self {
        Object { value }
    }

    #[uniffi::method(name = "renamed_method")]
    pub fn method(&self) -> i32 {
        self.value
    }
}
```

### In the bindings

```python
# Python
record = RenamedRecord(item=42)
result = renamed_function(record)
obj = RenamedObject.renamed_constructor(123)
value = obj.renamed_method()
```

```kotlin
// Kotlin
val record = RenamedRecord(item = 42)
val result = renamedFunction(record)
val obj = RenamedObject.renamedConstructor(123)
val value = obj.renamedMethod()
```

```swift
// Swift
let record = RenamedRecord(item: 42)
let result = renamedFunction(record: record)
let obj = RenamedObject.renamedConstructor(value: 123)
let value = obj.renamedMethod()
```