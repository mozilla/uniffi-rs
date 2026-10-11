---

## ✅ Aligned — Moving to Implementation

Thanks @pronebird @bendk — the direction is clear. Here's what I'll implement:

---

### 🎯 Final Design (Agreed by Both)

| Attribute | Generated Code |
|-----------|----------------|
| `#[export(async_runtime = "tokio")]` | Auto-enter shared runtime (requires `set_tokio_runtime`) |
| No attribute | Runs on foreign executor (current behavior) |

**Zero cost for non-users** — no `OnceLock` call, no branch, no code generated without the attribute. 🚀

---

### 📦 Implementation Files

```
uniffi_core/src/runtime.rs          # NEW: OnceLock<Arc<Runtime>> + set/get
uniffi_core/src/lib.rs              # pub mod runtime
uniffi_macros/src/export/scaffolding.rs  # Generate guard ONLY when async_runtime = "tokio"
fixtures/futures/src/lib.rs         # Tests: with/without attribute, block_in_place, multi-crate
docs/manual/src/futures.md          # Documentation
```

---

### 🔧 Scaffolding Logic (Exact Pattern)

```rust
// In gen_fn_scaffolding — ONLY when async_runtime = "tokio"
if matches!(ar, Some(AsyncRuntime::Tokio(_))) {
    future_expr = quote! {
        {
            let rt = ::uniffi::runtime::tokio_runtime()
                .expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup");
            async move {
                let _guard = rt.enter();  // Guard lives DURING poll (inside async block)
                #future_expr.await
            }
        }
    };
}
// No attribute → future_expr unchanged, runs on foreign executor
```

---

### ✅ What This Achieves

| Concern | Status |
|---------|--------|
| @bendk: "no cost for non-users" | ✅ Zero code without attribute |
| @pronebird: "share single runtime across crates" | ✅ Global `OnceLock<Arc<Runtime>>` in `uniffi_core` |
| @pronebird: "auto-enter for all async methods" | ✅ Automatic when opted in |
| Backwards compat | ✅ Existing code unchanged |
| `block_in_place` works | ✅ User's multi-threaded runtime |

---

### 🚀 Next Step

Implementing this exact design now. Will open a fresh PR with tests and docs.

One tiny decision: **panic with clear message** (Option A) vs `Result` (Option B) when runtime not configured:

```rust
// Option A (current plan) — panic with clear message
.expect("Tokio runtime not configured; call uniffi::set_tokio_runtime() at startup")

// Option B — return Result (more ergonomic but changes signature)
```

Going with **Option A** — matches `async_compat` behavior, keeps generated code simple, failure is a setup bug caught at first call.

---

On it! 🛠️