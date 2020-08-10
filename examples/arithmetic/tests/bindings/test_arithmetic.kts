import uniffi.arithmetic.*;

assert(add(2, 4) == 6L)
assert(add(4, 8) == 12L)

try {
    sub(0, 2)
    throw RuntimeException("Should have thrown a IntegerOverflow exception!")
} catch (e: ArithmeticErrorException) {
    // It's okay!
}

assert(sub(4, 2) == 2L)
assert(sub(8, 4) == 4L)

assert(equal(2, 2))
assert(equal(4, 4))

assert(!equal(2, 4))
assert(!equal(4, 8))
