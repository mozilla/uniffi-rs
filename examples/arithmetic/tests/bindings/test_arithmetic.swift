import arithmetic

do {
    let _ = try add(a: 18446744073709551615, b: 1)
    fatalError("Should have thrown a IntegerOverflow exception!")
} catch ArithmeticError.IntegerOverflow {
    // It's okay!
}

assert(add(a: 2, b: 4) == 6, "add work")
assert(add(a: 4, b: 8) == 12, "add work")

do {
    let _ = try sub(a: 0, b: 1)
    fatalError("Should have thrown a IntegerOverflow exception!")
} catch ArithmeticError.IntegerOverflow {
    // It's okay!
}

assert(sub(a: 4, b: 2) == 2, "sub work")
assert(sub(a: 8, b: 4) == 4, "sub work")

assert(equal(a: 2, b: 2), "equal works")
assert(equal(a: 4, b: 4), "equal works")

assert(!equal(a: 2, b: 4), "non-equal works")
assert(!equal(a: 4, b: 8), "non-equal works")
