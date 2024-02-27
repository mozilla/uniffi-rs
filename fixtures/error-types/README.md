Tests for objects as errors.

This works well with explicit type wrangling:

```rust
fn oops() -> Result<(), Arc<MyError>> {
    let e = anyhow::Error::msg("oops");
    Err(Arc::new(e.into()))
}
```

But a goal is to allow:

```rust
// doesn't work
fn oops() -> anyhow::Result<()> {
    anyhow::bail!("oops");
}
```

# Stuck!

the above uniffi expands to:

```rust
extern "C" fn uniffi_uniffi_error_types_fn_func_oops(
    call_status: &mut ::uniffi::RustCallStatus,
) -> <::std::result::Result<
    (),
    std::sync::Arc<ErrorInterface>,
> as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
...
    ::uniffi::rust_call(
        call_status,
        || {
            <::std::result::Result<(), std::sync::Arc<ErrorInterface>> as ::uniffi::LowerReturn ...>::lower_return(
                match uniffi_lift_args() {
                    Ok(uniffi_args) => oops().map_err(::std::convert::Into::into),
                    Err((arg_name, anyhow_error)) => ...
                },
            )
        },
    )
}
```

# Problem is:
```rust
                    Ok(uniffi_args) => oops().map_err(::std::convert::Into::into),
```

map_err has `anyhow::Error<>`, we want `Arc<ErrorInterface>`, `::into` can't do that.

This works for enum because  all `Arc<ErrorInterface>`s above are `ErrorEnum`. So above call is more like:
map_err has `CrateInternalError`, we want `CratePublicError`, `::into` can do that.
