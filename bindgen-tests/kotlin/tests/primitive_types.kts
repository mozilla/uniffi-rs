import uniffi.uniffi_bindgen_tests.*

// Test passing arguments from Kotlin to Rust
inputU8(42.toUByte())
inputI8((-42).toByte())
inputU16(42.toUShort())
inputI16((-42).toShort())
inputU32(42u)
inputI32(-42)
inputU64(42uL)
inputI64(-42L)
inputF32(0.5f)
inputF64(-3.5)
inputBool(true)
inputString("ABC")
// Test returning values from Rust to Kotlin
assert(outputU8() == 1.toUByte())
assert(outputI8() == 1.toByte())
assert(outputU16() == 1.toUShort())
assert(outputI16() == 1.toShort())
assert(outputU32() == 1u)
assert(outputI32() == 1)
assert(outputU64() == 1uL)
assert(outputI64() == 1L)
assert(outputF32() == 1.0f)
assert(outputF64() == 1.0)
assert(outputBool() == true)
assert(outputString() == "test-string")
// Test passing values both ways
assert(roundtripU8(42.toUByte()) == 42.toUByte())
assert(roundtripI8((-42).toByte()) == (-42).toByte())
assert(roundtripU16(42.toUShort()) == 42.toUShort())
assert(roundtripI16((-42).toShort()) == (-42).toShort())
assert(roundtripU32(42u) == 42u)
assert(roundtripI32(-42) == -42)
assert(roundtripU64(42uL) == 42uL)
assert(roundtripI64(-42L) == -42L)
assert(roundtripF32(0.5f) == 0.5f)
assert(roundtripF64(-3.5) == -3.5)
assert(roundtripBool(true) == true)
assert(roundtripString("ABC") == "ABC")
// Test calling a function with lots of args
// This function will sum up all the numbers, then negate the value since we passed in `true`
assert(sumWithManyTypes(
    1.toUByte(),
    (-2).toByte(),
    3.toUShort(),
    (-4).toShort(),
    5u,
    -6,
    7uL,
    -8L,
    9.5f,
    -10.5,
    true
) == 5.0)
