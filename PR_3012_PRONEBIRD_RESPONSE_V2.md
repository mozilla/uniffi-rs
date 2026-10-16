---

## Aligned on Global Runtime + Opt-In via `async_runtime = "tokio"`

That's exactly the right balance. Here's the refined approach:

---

### Behavior

| Attribute | Generated Code |
|-----------|----------------|
| `#[export(async_runtime = "tokio")]` | Auto-enter shared runtime (require configured) |
| No attribute | Runs on foreign executor (current behavior) |

---

### Implementation

```rust
// scaffolding.rs - in gen_fn_scaffolding
let mut future_expr = rust_fn_call;

if matches!(ar, Some(AsyncRuntime::Tokio(_))) {
    // User explicitly opted in → generate runtime enter guard
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

// No async_runtime → future_expr unchanged, runs on foreign executor
```

---

### What This Achieves

| Concern | Addressed |
|---------|-----------|
| @bendk: "no cost for non-users" | ✅ Zero code generated without `async_runtime = "tokio"` |
| @pronebird: "share single runtime across crates" | ✅ Global `OnceLock<Arc<Runtime>>` in `uniffi_core` |
| @pronebird: "auto-enter for all async methods" | ✅ Automatic when opted in |
| Backwards compat | ✅ Existing code unchanged |
| `block_in_place` works | ✅ User's multi-threaded runtime |

---

### Updated Files

```
uniffi_core/src/runtime.rs      # NEW: OnceLock<Arc<Runtime>> + set/get
uniffi_core/src/lib.rs          # pub mod runtime
uniffi_macros/src/export/scaffolding.rs  # Generate guard ONLY when async_runtime = "tokio"
fixtures/futures/src/lib.rs     # Test: with/without attribute, block_in_place, multi-crate
docs/manual/src/futures.md      # Document the attribute + set_tokio_runtime
```

---

### Next Step

If this aligns, I'll implement. One question:

**Error message** when runtime not configured:
```rust
.expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup")
```

Or return `Result` in generated code? (More ergonomic but adds error handling)

Otherwise, ready to code. 👍