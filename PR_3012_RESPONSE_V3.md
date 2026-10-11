---

## Response to @pronebird

Fair point. You're right — crate-level `setup_scaffolding!(tokio_runtime = ...)` means each crate needs its own config. That's not what you want.

**What you want:** Single global runtime, configured once, shared across all crates in the workspace.

**That's exactly what the `OnceLock` approach does** — I just explained it poorly. Let me be concrete:

---

## The Actual Solution: Global Runtime in `uniffi_core`

```rust
// uniffi_core/src/runtime.rs (new file)
use std::sync::OnceLock;
use tokio::runtime::Handle;

static TOKIO_RUNTIME: OnceLock<Handle> = OnceLock::new();

/// Call ONCE at app startup (from any crate)
pub fn set_tokio_runtime(handle: Handle) -> Result<(), AlreadySetError> {
    TOKIO_RUNTIME.set(handle).map_err(|_| AlreadySetError)
}

/// Internal: used by generated scaffolding
pub(crate) fn tokio_runtime_handle() -> Option<Handle> {
    TOKIO_RUNTIME.get().cloned()
}
```

**User code (ONE crate, ONCE at startup):**
```rust
// my_app/src/main.rs or lib.rs
use tokio::runtime::Builder;
use uniffi::set_tokio_runtime;

fn init() {
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-app-tokio")
        .build()
        .unwrap();
    
    set_tokio_runtime(rt.handle().clone()).expect("runtime already set");
}
```

**Generated scaffolding (automatic, per-function):**
```rust
// In scaffolding.rs - generated for EVERY exported async fn
let fut = async { foo().await };

// ONLY if runtime was configured:
if let Some(handle) = uniffi::runtime::tokio_runtime_handle() {
    let _guard = handle.enter();
    rust_future_new(fut, ...)
} else {
    rust_future_new(fut, ...)  // no enter guard, runs on foreign executor
}
```

---

## Why This Works Across Crates

| Crate | `setup_scaffolding!` | Generated Code |
|-------|---------------------|----------------|
| `crate_a` | No param | Checks `uniffi::runtime::tokio_runtime_handle()` → uses shared runtime |
| `crate_b` | No param | Same check → same shared runtime |
| `crate_c` | No param | Same check → same shared runtime |

**No per-crate config needed.** The `OnceLock` lives in `uniffi_core` — single instance per process (since workspace = single `uniffi` version).

---

## Zero Cost When Unused

```rust
// If NO crate calls set_tokio_runtime():
// - OnceLock is empty
// - tokio_runtime_handle() returns None
// - Generated code: just `rust_future_new(fut, ...)` — no branch, no enter, no OnceLock access
// - Compiles to exact same code as today
```

The `OnceLock::get()` is only hit when a runtime IS configured. Non-users pay nothing.

---

## What This Solves

| Your Requirement | Fixed? |
|------------------|--------|
| Single runtime shared across all crates | ✅ — `OnceLock` in `uniffi_core` |
| Configure once at startup | ✅ — `set_tokio_runtime(handle)` |
| No "one-liner crate" boilerplate | ✅ — call from any crate's init |
| `block_in_place` works | ✅ — runtime is multi-threaded |
| Zero cost for non-users | ✅ — no runtime = no generated code |
| Backwards compat | ✅ — existing `async_runtime = "tokio"` unchanged |

---

## Implementation Surface (Small)

```
uniffi_core/
├── src/
│   ├── runtime.rs      # NEW ~80 lines: OnceLock + set_tokio_runtime + tokio_runtime_handle
│   └── lib.rs          # pub use runtime::*;
uniffi_macros/
└── src/export/
    └── scaffolding.rs  # Add automatic .enter() guard when handle exists
fixtures/futures/
└── src/lib.rs          # Test: multi-crate, block_in_place, thread names
docs/manual/src/futures.md  # Document set_tokio_runtime
```

---

## Next Step

If this global-runtime approach matches what you want, I'll implement it. No per-crate config, no manual wrapping, no AI summaries.

Just:
```rust
// Once at startup
uniffi::set_tokio_runtime(my_rt.handle().clone());

// Every async export in EVERY crate just works
#[uniffi::export]
async fn foo() { ... }
```

Sound right? 👍