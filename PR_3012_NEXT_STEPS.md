---

## Moving Forward — Implementation Plan

Agreed on the approach: **global runtime + opt-in via `async_runtime = "tokio"`**. 

Here's what I'll implement:

---

### 1. `uniffi_core/src/runtime.rs` (new)
```rust
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

pub fn set_tokio_runtime(rt: Runtime) -> Result<(), AlreadySetError> {
    TOKIO_RUNTIME.set(Arc::new(rt)).map_err(|_| AlreadySetError)
}

pub(crate) fn tokio_runtime() -> Option<Arc<Runtime>> {
    TOKIO_RUNTIME.get().cloned()
}

pub fn shutdown_tokio_runtime() {
    if let Some(rt) = TOKIO_RUNTIME.get() {
        rt.shutdown_background();
    }
}
```

---

### 2. `uniffi_macros/src/export/scaffolding.rs` — modify `gen_fn_scaffolding`
```rust
// Only when user writes: #[uniffi::export(async_runtime = "tokio")]
if matches!(ar, Some(AsyncRuntime::Tokio(_))) {
    future_expr = quote! {
        {
            let rt = ::uniffi::runtime::tokio_runtime()
                .expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup");
            async move {
                let _guard = rt.enter();
                #future_expr.await
            }
        }
    };
}
```

---

### 3. Tests in `fixtures/futures/src/lib.rs`
```rust
#[uniffi::export(async_runtime = "tokio")]
pub async fn block_in_place_works() -> String {
    tokio::task::block_in_place(|| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        "ok".to_string()
    })
}
```

---

### 4. Docs in `docs/manual/src/futures.md`

---

### One Decision Needed

**Error handling** when runtime not configured:

```rust
// Option A: panic with clear message (current plan)
.expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup")

// Option B: return Result from generated code (more ergonomic, but changes signature)
```

I'll go with **Option A** (panic) — matches `async_compat` behavior, keeps generated code simple, failure is a setup bug caught at first call.

---

**Ready to implement.** Will open a fresh PR with the above. 👍