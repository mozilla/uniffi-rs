import uniffi.uniffi_bindgen_tests.*

// Kotlin uses the display impl for `toString`
assert(RustTraitTest(1, 2).toString() == "display-test-string")

// Kotlin falls back to the debug impl for `Display` is not exported
assert(RustTraitTest2(1, 2).toString() == "debug-test-string")

// The Rust code only uses `a` for the equality
assert(RustTraitTest(1, 2) == RustTraitTest(1, 3))
assert(RustTraitTest(1, 2) != RustTraitTest(2, 2))


// The Rust code only uses `a` for the ordering
assert(RustTraitTest(1, 2) < RustTraitTest(2, 3))
assert(RustTraitTest(1, 2) <= RustTraitTest(1, 3)
    && RustTraitTest(1, 2) >= RustTraitTest(1, 3))

// The Rust code only uses `a` for the hash
assert(RustTraitTest(1, 2).hashCode() == RustTraitTest(1, 3).hashCode())
assert(RustTraitTest(2, 2).hashCode() != RustTraitTest(1, 2).hashCode())
