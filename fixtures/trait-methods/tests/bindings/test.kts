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

assert(TraitEnum.N.toString() == "TraitEnum::N")
assert((TraitEnum::S)("hello").toString() == "TraitEnum::S(\"hello\")")
assert((TraitEnum::I)(1).toString() == "TraitEnum::I(1)")
assert((UdlEnum::S)("hello").toString() == "S { s: \"hello\" }")

assert((UdlEnum::S)("hello") == (UdlEnum::S)("hello"))
assert((UdlEnum::S)("hello") == (UdlEnum::S)("other"))
assert((UdlEnum::S)("hello") < (UdlEnum::I)(0))
assert((TraitEnum::I)(1) == (TraitEnum::I)(1))
assert((TraitEnum::I)(1) == (TraitEnum::I)(2))
assert((TraitEnum::S)("hello") < (TraitEnum::I)(0))

// flat enum with Display only - Kotlin doesn't support Eq/Ord/Hash exports for flat enums
val flatAlpha = getFlatTraitEnum(0u)
assert(flatAlpha == FlatTraitEnum.ALPHA)
assert(getFlatTraitEnum(1u) > flatAlpha)
assert(flatAlpha.toString() == "FlatTraitEnum::flat-alpha")
assert(getFlatTraitEnum(2u).toString() == "FlatTraitEnum::flat-gamma")
val flatSet = setOf(flatAlpha)
assert(flatSet.contains(FlatTraitEnum.ALPHA))
assert(!flatSet.contains(FlatTraitEnum.BETA))

// Errors

// flat error - Display export on flat errors
try {
    throwFlatError(0u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.NotFound) {
    assert(e.toString() == "error: not found")
}

// error with multiple exported traits (Debug, Display, Eq, Ord, Hash)
try {
    throwMultipleTraitError(0u)
    throw AssertionError("should have thrown")
} catch (e: MultipleTraitException.NoData) {
    assert(e.toString() == "MultipleTraitError::NoData")
}

// test nested error with Display
try {
    throwMultipleTraitError(1u)
    throw AssertionError("should have thrown")
} catch (e: MultipleTraitException.Nested) {
    assert(e.toString() == "nested error: error: not found")
}

// test Eq on flat variants
var err1: MultipleTraitException? = null
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    err1 = e
}
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    assert(err1 == e)
}

// test Ord
var err2: MultipleTraitException? = null
try {
    throwMultipleTraitError(1u)
} catch (e: MultipleTraitException.Nested) {
    err2 = e
}
assert(err1!! < err2!!)

// error that doesn't end in "Error" - tests class_name filter doesn't break non-Error types
try {
    throwApiFailure(0u)
    throw AssertionError("should have thrown")
} catch (e: ApiFailure.NetworkIssue) {
    assert(e.toString() == "api network issue")
}

// test Eq on non-Error type
var apiErr: ApiFailure? = null
try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    apiErr = e
}
try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    assert(apiErr == e)
}

// test Ord on non-Error type
var apiErr2: ApiFailure? = null
try {
    throwApiFailure(1u)
} catch (e: ApiFailure.ServerDown) {
    apiErr2 = e
}
assert(apiErr!! < apiErr2!!)
