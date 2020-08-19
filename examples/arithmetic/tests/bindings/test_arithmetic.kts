import uniffi.arithmetic.*;

assert(add(2u, 4u) == 6uL)
assert(add(4u, 8u) == 12uL)

try {
    sub(0u, 2u)
    throw RuntimeException("Should have thrown a IntegerOverflow exception!")
} catch (e: ArithmeticErrorException) {
    // It's okay!
}

assert(sub(4u, 2u) == 2uL)
assert(sub(8u, 4u) == 4uL)

assert(equal(2u, 2uL))
assert(equal(4u, 4uL))

assert(!equal(2u, 4uL))
assert(!equal(4u, 8uL))
