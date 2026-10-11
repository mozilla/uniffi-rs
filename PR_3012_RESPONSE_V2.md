---

## Response to Latest Feedback

Thanks @pronebird @bendk — this clarifies a lot. Let me address the confusion and propose the direction that matches what you're both describing.

---

## The Core Misunderstanding

My previous proposal had **users manually wrapping** calls in `uniffi::tokio_spawn()`. That's not what you want.

**What you want (crate-level, automatic):**
```rust
// User code — ONE line in crate root
uniffi::setup_scaffolding!(tokio_runtime = MY_RUNTIME);

// Every exported async fn in this crate AUTOMATICALLY runs with runtime entered
#[uniffi::export]
async fn foo() -> String {
    tokio::time::sleep(...).await;  // just works
    tokio::spawn(...);              // just works
    block_in_place(|| ...);         // just works
}
```

No `tokio_spawn` wrapper. No manual `.enter()`. The proc macro generates the enter guard per-function.

---

## @bendk — Addressing Your Two Points

### 1. "Are users expected to write that code block? What's the UniFFI feature?"

**No, they don't write it.** The feature is: **crate-level opt-in via `setup_scaffolding!`**

```rust
// Generated per-function scaffolding (simplified)
pub extern "C" fn my_crate_foo() -> Handle {
    let fut = async {
        // User's async fn body
        foo().await
    };
    
    // ONLY generated if crate opted in via setup_scaffolding!(tokio_runtime = ...)
    let _guard = crate::UNIFFI_TOKIO_RUNTIME.enter();
    
    rust_future_new(fut, crate::UniFfiTag)
}
```

The proc macro checks for a `const` emitted by `setup_scaffolding!` and injects the `.enter()` guard automatically.

### 2. "Acquiring OnceLock — want to avoid cost for users who don't need this"

**Agreed — zero cost for non-users.** Here's how:

```rust
// setup_scaffolding! emits THIS (only when tokio_runtime param provided):
pub const UNIFFI_TOKIO_RUNTIME: Option<&'static Runtime> = Some(&MY_RUNTIME);

// Per-function scaffolding reads it:
#[cfg(feature = "tokio")]
let _guard = crate::UNIFFI_TOKIO_RUNTIME.map(|rt| rt.enter());

// If no tokio_runtime param → const is None or doesn't exist → zero code generated
// No OnceLock, no runtime lookup, no branch at runtime
```

**Cost for non-users:** Literally zero. No `OnceLock`, no global, no check. The generated code simply doesn't include the enter guard.

---

## @pronebird — "All async methods should simply enter runtime"

**Yes, but only for crates that opt in.** The `tokio_runtime` param on `setup_scaffolding!` is the opt-in signal.

If crate A has `tokio_runtime = RT` → all its exported async fns get `.enter(RT)` guard.
If crate B has no param → no guard, runs on foreign executor (current behavior).

This also solves your multi-crate sharing — both crates point to same `MY_RUNTIME`.

---

## Revised Implementation Plan

| File | Change |
|------|--------|
| `uniffi_macros/src/export/attributes.rs` | Add `tokio_runtime: Option<Expr>` to `ScaffoldingArgs` |
| `uniffi_macros/src/setup_scaffolding.rs` | Parse `tokio_runtime = expr`, emit `const UNIFFI_TOKIO_RUNTIME: Option<&'static Runtime> = Some(&expr)` |
| `uniffi_macros/src/export/scaffolding.rs` | In `gen_fn_scaffolding`: if `crate::UNIFFI_TOKIO_RUNTIME` is `Some`, wrap future in `.enter()` |
| `uniffi_core/src/lib.rs` | Re-export `Runtime`/`Handle` types for user convenience (optional) |
| `fixtures/futures/src/lib.rs` | Test: crate with `tokio_runtime` param, verify `block_in_place` works |
| `docs/manual/src/futures.md` | Document the new param |

**Key insight:** The `const Option<&'static Runtime>` pattern lets the proc macro:
- Generate the enter guard **only when the const is `Some`**
- Be **completely erased** when not provided (no runtime check, no OnceLock)
- Work **across crates** — each crate's `setup_scaffolding!` emits its own const pointing to the same shared `Runtime`

---

## What This Solves

| Problem | Fixed? |
|---------|--------|
| `block_in_place` panic | ✅ — crate opts into multi-threaded runtime |
| Configure thread count/names | ✅ — user builds `Runtime` once |
| Share runtime with app | ✅ — pass `app_runtime.handle()` |
| Multi-crate workspace | ✅ — all crates point to same `Runtime` |
| Zero cost for non-users | ✅ — no const = no generated code |
| Backwards compat | ✅ — existing code unchanged |

---

## Open Questions (Still Need Your Call)

1. **Const type**: `Option<&'static Runtime>` vs `Option<Handle>` — `Handle` is cheaper to clone, but `&'static Runtime` lets us call `.enter()` directly. Leaning `Handle` since `Handle::enter()` exists.

2. **`setup_scaffolding!` param name**: `tokio_runtime = expr` or `tokio = expr`? The latter matches the feature flag name.

3. **Error if const missing**: If user writes `tokio_runtime = ...` but forgets `setup_scaffolding!`, proc macro should emit clear error.

4. **Per-function override**: `#[uniffi::export(tokio_runtime = ...)]`? You said no strong feelings — I'll skip unless requested.

---

## Next Step

If this crate-level automatic-enter approach matches what you want, I'll:
1. Implement the macro plumbing (setup_scaffolding → scaffolding.rs communication)
2. Add fixture test with `block_in_place`
3. Update docs

**No manual `tokio_spawn` wrapper. No OnceLock for non-users. Pure compile-time opt-in.**

Sound right? 👍