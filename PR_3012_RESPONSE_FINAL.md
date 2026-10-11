---

## Corrections to Previous Response

Two critical fixes needed:

---

### 1. Enter Guard Must Be Inside Async Block (Not Outside)

```rust
// WRONG (from my previous response):
if let Some(handle) = uniffi::runtime::tokio_runtime_handle() {
    let _guard = handle.enter();  // Dropped BEFORE polling!
    rust_future_new(fut, ...)
}

// CORRECT (guard lives during POLL):
let fut = {
    let rt = uniffi::runtime::tokio_runtime();  // Option<Arc<Runtime>>
    async move {
        if let Some(rt) = rt {
            let _guard = rt.enter();  // Active during .await
            foo().await
        } else {
            foo().await
        }
    }
};
rust_future_new(fut, crate::UniFfiTag)
```

---

### 2. Store `Arc<Runtime>` Not `Handle` (Ownership)

```rust
// uniffi_core/src/runtime.rs
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

pub fn set_tokio_runtime(rt: Runtime) -> Result<(), AlreadySetError> {
    TOKIO_RUNTIME.set(Arc::new(rt)).map_err(|_| AlreadySetError)
}

pub(crate) fn tokio_runtime() -> Option<Arc<Runtime>> {
    TOKIO_RUNTIME.get().cloned()
}

// Optional: graceful shutdown
pub fn shutdown_tokio_runtime() {
    if let Some(rt) = TOKIO_RUNTIME.get() {
        rt.shutdown_background();
    }
}
```

User passes owned `Runtime`, UniFFI keeps it alive. No lifetime footguns.

---

## What Stays Same (Already Covered)

- Global `OnceLock` in `uniffi_core` → single runtime shared across all crates
- Configure once: `uniffi::set_tokio_runtime(my_rt)` at startup
- Auto-enter for all exported async fns in all crates
- `block_in_place` works, zero cost when unused, backwards compat

---

## Updated Implementation Surface

```
uniffi_core/src/runtime.rs      # NEW ~60 lines (Arc<Runtime> + set/get)
uniffi_core/src/lib.rs          # pub mod runtime
uniffi_macros/src/export/scaffolding.rs  # Generate async block with .enter()
fixtures/futures/src/lib.rs     # Test: multi-crate, block_in_place, shutdown
docs/manual/src/futures.md      # Document set_tokio_runtime + shutdown
```

---

## Next Step

If this approach works, I'll implement. Ready when you are. 👍