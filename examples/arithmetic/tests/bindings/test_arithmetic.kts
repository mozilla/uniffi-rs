import uniffi.arithmetic.*;

assert(add(2, 3, Overflow.SATURATING) == 5L)

assert(equal(4, 4))
assert(!equal(4, 5))
