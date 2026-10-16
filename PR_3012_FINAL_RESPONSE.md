---

## PR #3012: Global Tokio Runtime Configuration - Ready for Review

### 🎯 Summary
Implements the global Tokio runtime configuration from #1726 (per @pronebird's proposal with 7 👍). Supersedes #2992.

### 🔗 Links
- **PR**: https://github.com/mozilla/uniffi-rs/pull/3012
- **Implements**: #1726
- **Supersedes**: #2992 (closed)

### ✅ Implementation Complete

| Component | Status |
|-----------|--------|
| `uniffi_core/src/runtime.rs` | ✅ New module with `set_tokio_runtime()`, `tokio_runtime_compat()`, `shutdown_tokio_runtime()` |
| `uniffi_macros/src/export/scaffolding.rs` | ✅ Generates `tokio_runtime_compat()` wrapper |
| `uniffi_core/Cargo.toml` | ✅ Updated `async-compat` to 0.2.6, added `tokio/rt-multi-thread` |
| `fixtures/futures/src/lib.rs` | ✅ Tests for `block_in_place`, `spawn`, `thread_name` |
| `fixtures/futures/tests/bindings/test_futures.py` | ✅ Python tests for new functions |
| `docs/manual/src/futures.md` | ✅ Documentation for custom runtime |
| `CHANGELOG.md` | ✅ Release notes added |

### ✅ Test Results
- **Python tests**: 20/20 passing (including new tests)
- **Core Rust tests**: All pass (uniffi_core, uniffi_macros, uniffi, uniffi_meta, uniffi_pipeline, uniffi_bindgen)
- **Kotlin/Swift/Ruby**: Skipped (toolchains not in CI)

### 🎯 Key Features
- **Global runtime**: `uniffi::set_tokio_runtime(rt)` - configure once
- **Auto-enter**: `async_runtime = "tokio"` functions automatically use shared runtime
- **Zero cost**: No runtime code generated without `async_runtime = "tokio"`
- **Backwards compat**: Existing code works via `async_compat` fallback
- **Send-safe**: `TokioRuntimeCompat` enters runtime during polling, not across `.await`

### 📝 Usage Example
```rust
// At startup (once)
uniffi::set_tokio_runtime(
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-app-tokio")
        .build()
        .unwrap()
).unwrap();

// In UniFFI code
#[uniffi::export(async_runtime = "tokio")]
pub async fn my_fn() -> String {
    tokio::task::block_in_place(|| "done".to_string()) // Works!
}
```

### 👀 Ready for Review
@bendk @pronebird @mhammond @Hywan @jplatte — implementation matches the agreed design. Ready for your review!

---

*Implementation commits: `6ef57fd05` (feat: add global Tokio runtime configuration)*