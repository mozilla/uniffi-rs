import uniffi.trait_methods.*

val m = TraitMethods("yo")
assert(m.toString() == "TraitMethods(yo)")

assert(m == TraitMethods("yo"))
assert(m != TraitMethods("yoyo"))

val map = mapOf(m to 1, TraitMethods("yoyo") to 2)
assert(map[m] == 1)
assert(map[TraitMethods("yoyo")] == 2)

assert(ProcTraitMethods("a") < ProcTraitMethods("b"))
assert(m < TraitMethods("z"))
assert(m <= TraitMethods("z"))
assert(TraitMethods("z") > m)

// Records
assert(UdlRecord(s="yo", i=2) == UdlRecord(s="yo", i=2))
assert(UdlRecord(s="yo", i=2) == UdlRecord(s="yo", i=3))
assert(UdlRecord(s="hi", i=2) != UdlRecord(s="yo", i=3))
assert(UdlRecord(s="a", i=2) < UdlRecord(s="yo", i=1))

assert(TraitRecord(s="yo", i=2).toString() == "TraitRecord { s: \"yo\", i: 2 }")
assert(TraitRecord(s="yo", i=2) == TraitRecord(s="yo", i=2))
assert(TraitRecord(s="yo", i=2) == TraitRecord(s="yo", i=3))
assert(TraitRecord(s="hi", i=2) != TraitRecord(s="yo", i=3))
assert(TraitRecord(s="a", i=2) < TraitRecord(s="yo", i=1))

// Enums

assert((UdlEnum::S)("hello") == (UdlEnum::S)("hello"))
assert((UdlEnum::S)("hello") == (UdlEnum::S)("other"))
assert((UdlEnum::S)("hello") < (UdlEnum::I)(0))
assert((TraitEnum::I)(1) == (TraitEnum::I)(1))
assert((TraitEnum::I)(1) == (TraitEnum::I)(2))
assert((TraitEnum::S)("hello") < (TraitEnum::I)(0))

// nested enums with payloads and Display trait
assert((TraitEnum::S)("hello").toString() == "TraitEnum::S(\"hello\")")
assert((TraitEnum::I)(1).toString() == "TraitEnum::I(1)")
assert((UdlEnum::S)("hello").toString() == "S { s: \"hello\" }")

// flat enum (no payload) with Display
assert(getEnumWithDisplayExport(0u).toString() == "display: One")
assert(getEnumWithDisplayExport(1u).toString() == "display: Two")
assert(getEnumWithDisplayExport(2u).toString() == "display: Three")

// nested enum with another enum (that has Display) as payload
assert(getNestedEnumWithDisplay(0u).toString() == "nested simple: display: One")
assert(getNestedEnumWithDisplay(1u).toString() == "nested complex [test]: display: Two")
assert(getNestedEnumWithDisplay(2u).toString() == "nested simple: display: Three")

// flat enum with Eq/Ord/Hash - uses Kotlin enum class's built-in support
val flatAlpha = getFlatTraitEnum(0u)
assert(flatAlpha == FlatTraitEnum.ALPHA)
assert(getFlatTraitEnum(1u) > flatAlpha)
assert(flatAlpha.toString() == "FlatTraitEnum::flat-alpha")
assert(getFlatTraitEnum(2u).toString() == "FlatTraitEnum::flat-gamma")
val flatSet = setOf(flatAlpha)
assert(flatSet.contains(FlatTraitEnum.ALPHA))
assert(!flatSet.contains(FlatTraitEnum.BETA))

// flat enum with explicit numeric discriminant and Display
val numericRed = getNumericEnum(100u)
assert(numericRed == NumericEnum.RED)
assert(numericRed.value == 100u.toUShort())
assert(numericRed.toString() == "NumericEnum::color-red")
assert(getNumericEnum(200u).toString() == "NumericEnum::color-green")
assert(getNumericEnum(300u).toString() == "NumericEnum::color-blue")
assert(getNumericEnum(999u).value == 300u.toUShort())  // defaults to Blue

// Errors

// flat error with Display
try {
    throwTraitError(0u)
    throw AssertionError("should have thrown")
} catch (e: FlatErrorWithDisplayExport.TooMany) {
    assert(e.toString() == "display: too many items: 100")
}

try {
    throwTraitError(1u)
    throw AssertionError("should have thrown")
} catch (e: FlatErrorWithDisplayExport.TooFew) {
    assert(e.toString() == "display: too few items: 0")
}

// nested error with another error (that has Display) as payload
try {
    throwNestedError(0u)
    throw AssertionError("should have thrown")
} catch (e: NestedErrorWithDisplay.Simple) {
    assert(e.toString() == "nested simple error: display: too many items: 42")
}

try {
    throwNestedError(1u)
    throw AssertionError("should have thrown")
} catch (e: NestedErrorWithDisplay.Complex) {
    assert(e.toString() == "nested complex error [nested]: display: too few items: 7")
}

try {
    throwNestedError(2u)
    throw AssertionError("should have thrown")
} catch (e: NestedErrorWithDisplay.Simple) {
    assert(e.toString() == "nested simple error: display: too few items: 0")
}
