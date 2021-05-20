# Getting started

Say your company has a simple `math` crate with the following `lib.rs`:

```rust
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

And top brass would like you to expose this *business-critical* operation to Kotlin and Swift.  
**Don't panic!** We will show you how do that using UniFFI.
