Ensure that generated bindings are compatible with crates which have a
local

```rust
pub enum MyError { ... };
pub type Result<T> = std::result::Result<T, MyError>
```
