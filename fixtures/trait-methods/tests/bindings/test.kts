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

// flat enum with Display only - Kotlin doesn't support Eq/Ord/Hash exports for flat enums
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

// mixed enum with object variants (requires parent-level trait impls - line 117)
// object variants have no fields, so they can't have their own trait impl bodies
// they MUST inherit from the sealed parent's implementations
val objectVariant1 = getMixedEnum(0u) // NoFields - object variant
val objectVariant2 = getMixedEnum(2u) // AnotherNoFields - object variant
val dataVariant1 = getMixedEnum(1u)   // WithString - data class variant
val dataVariant2 = getMixedEnum(3u)   // WithNumber - data class variant

// test toString on object variants - will fail without line 117
assert(objectVariant1.toString() == "MixedEnum::NoFields")
assert(objectVariant2.toString() == "MixedEnum::AnotherNoFields")

// test equals on object variants - will fail without line 117
assert(objectVariant1 == getMixedEnum(0u))
assert(objectVariant2 == getMixedEnum(2u))

// test hashCode on object variants - will fail without line 117
assert(objectVariant1.hashCode() == getMixedEnum(0u).hashCode())
val mixedSet = setOf(objectVariant1, objectVariant2)
assert(mixedSet.contains(getMixedEnum(0u)))

// test compareTo on object variants - will fail without line 117
assert(objectVariant1 < objectVariant2)
assert(objectVariant1 < dataVariant1)
assert(objectVariant2 < dataVariant2)

// test mixed list with object variants - will fail without line 117
val mixedList: List<MixedEnum> = listOf(
    dataVariant2,
    objectVariant2,
    dataVariant1,
    objectVariant1
)
val sortedMixed = mixedList.sorted()
assert(sortedMixed[0] is MixedEnum.NoFields) // object variant first
assert(sortedMixed[3] is MixedEnum.WithNumber) // data variant last

// Errors

// flat error - only toString() is generated (like flat enums)
try {
    throwFlatError(0u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.NotFound) {
    assert(e.toString() == "error: not found")
}

try {
    throwFlatError(1u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.Unauthorized) {
    assert(e.toString() == "error: unauthorized")
}

try {
    throwFlatError(2u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.InternalException) {
    assert(e.toString() == "error: internal error")
}

// error with fields
try {
    throwWithFieldsError(0u)
    throw AssertionError("should have thrown")
} catch (e: WithFieldsException.TooMany) {
    assert(e.toString() == "display: too many items: 100")
}

try {
    throwWithFieldsError(1u)
    throw AssertionError("should have thrown")
} catch (e: WithFieldsException.TooFew) {
    assert(e.toString() == "display: too few items: 0")
}

// nested error with another error (that has Display) as payload
try {
    throwNestedWithDisplayError(0u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Simple) {
    assert(e.toString() == "nested simple error: display: too many items: 42")
}

try {
    throwNestedWithDisplayError(1u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Complex) {
    assert(e.toString() == "nested complex error [nested]: display: too few items: 7")
}

try {
    throwNestedWithDisplayError(2u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Simple) {
    assert(e.toString() == "nested simple error: display: too few items: 0")
}

// mixed error with flat variants (no fields) and field variants
try {
    throwMixedFieldError(0u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.SimpleFailure) {
    assert(e.toString() == "error: simple failure")
}

try {
    throwMixedFieldError(1u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.ValidationFailed) {
    assert(e.toString() == "error: validation failed with 5 errors")
}

try {
    throwMixedFieldError(2u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.Timeout) {
    assert(e.toString() == "error: timeout")
}

try {
    throwMixedFieldError(3u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.InvalidInput) {
    assert(e.toString() == "error: invalid input: bad data")
}

// mixed error with all possible exported traits (Debug, Display, Eq, Ord, Hash)
// tests that flat variants inherit all trait implementations from parent
var flatVariant1: MultipleTraitException? = null
var flatVariant2: MultipleTraitException? = null
var fieldVariant1: MultipleTraitException? = null
var fieldVariant2: MultipleTraitException? = null

// catch exceptions to test them
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    flatVariant1 = e
}

try {
    throwMultipleTraitError(2u)
} catch (e: MultipleTraitException.AnotherFlat) {
    flatVariant2 = e
}

try {
    throwMultipleTraitError(1u)
} catch (e: MultipleTraitException.WithCode) {
    fieldVariant1 = e
}

try {
    throwMultipleTraitError(3u)
} catch (e: MultipleTraitException.WithMessage) {
    fieldVariant2 = e
}

// test Display (toString) on flat variants
assert(flatVariant1!!.toString() == "MultipleTraitError::NoData")
assert(flatVariant2!!.toString() == "MultipleTraitError::AnotherFlat")

// test Display (toString) on field variants
assert(fieldVariant1!!.toString() == "MultipleTraitError::WithCode(42)")
assert(fieldVariant2!!.toString() == "MultipleTraitError::WithMessage(test)")

// test Eq (equals) on flat variants
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    assert(flatVariant1 == e) // should be equal
}

try {
    throwMultipleTraitError(2u)
} catch (e: MultipleTraitException.AnotherFlat) {
    assert(flatVariant2 == e) // should be equal
}

// test Hash (hashCode) on flat variants
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    assert(flatVariant1!!.hashCode() == e.hashCode())
}

val errorSet = setOf(flatVariant1!!, flatVariant2!!)
try {
    throwMultipleTraitError(0u)
} catch (e: MultipleTraitException.NoData) {
    assert(errorSet.contains(e))
}

// test Ord (compareTo) on flat and field variants
assert(flatVariant1!! < flatVariant2!!)
assert(flatVariant1!! < fieldVariant1!!)
assert(flatVariant2!! < fieldVariant2!!)
assert(fieldVariant1!! < fieldVariant2!!)

// test sorting mixed error list
val errorList: List<MultipleTraitException> = listOf(
    fieldVariant2!!,
    flatVariant2!!,
    fieldVariant1!!,
    flatVariant1!!
)
val sortedErrors = errorList.sorted()
assert(sortedErrors[0] is MultipleTraitException.NoData) // flat variant first
assert(sortedErrors[3] is MultipleTraitException.WithMessage) // field variant last

// error that doesn't end in "Error" - should keep name as-is (ApiFailure, not ApiFailureException)
// tests that class_name filter doesn't break non-Error error types
var apiErr1: ApiFailure? = null
var apiErr2: ApiFailure? = null
var apiErr3: ApiFailure? = null

try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    apiErr1 = e
}

try {
    throwApiFailure(1u)
} catch (e: ApiFailure.Timeout) {
    apiErr2 = e
}

try {
    throwApiFailure(2u)
} catch (e: ApiFailure.ServerDown) {
    apiErr3 = e
}

// test Display (toString)
assert(apiErr1!!.toString() == "api network issue")
assert(apiErr2!!.toString() == "api timeout after 5000ms")
assert(apiErr3!!.toString() == "api server down")

// test Eq (equals) - should work with class_name filter
try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    assert(apiErr1 == e)
}

try {
    throwApiFailure(1u)
} catch (e: ApiFailure.Timeout) {
    assert(apiErr2 == e)
    assert(apiErr1 != e) // different variants
}

// test Hash (hashCode)
try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    assert(apiErr1!!.hashCode() == e.hashCode())
}

val apiSet = setOf(apiErr1!!, apiErr2!!)
try {
    throwApiFailure(0u)
} catch (e: ApiFailure.NetworkIssue) {
    assert(apiSet.contains(e))
}

// test Ord (compareTo) - should work with class_name filter
assert(apiErr1!! < apiErr2!!)
assert(apiErr2!! < apiErr3!!)

// test field comparison for Timeout variant (same variant, different field values)
var timeout1000: ApiFailure? = null
var timeout2000: ApiFailure? = null
var timeout5000: ApiFailure? = null
try {
    throwApiFailureTimeout(1000u)
} catch (e: ApiFailure.Timeout) {
    timeout1000 = e
}
try {
    throwApiFailureTimeout(2000u)
} catch (e: ApiFailure.Timeout) {
    timeout2000 = e
}
try {
    throwApiFailureTimeout(5000u)
} catch (e: ApiFailure.Timeout) {
    timeout5000 = e
}

assert(timeout1000!! < timeout2000!!)
assert(timeout2000!! < timeout5000!!)
assert(timeout1000!! < timeout5000!!)
assert(timeout1000!!.toString() == "api timeout after 1000ms")
assert(timeout5000!!.toString() == "api timeout after 5000ms")

// test field comparison for RateLimited variant (same variant, different field values)
var rateLimited30: ApiFailure? = null
var rateLimited60: ApiFailure? = null
var rateLimited120: ApiFailure? = null
try {
    throwApiFailureRateLimited(30u)
} catch (e: ApiFailure.RateLimited) {
    rateLimited30 = e
}
try {
    throwApiFailureRateLimited(60u)
} catch (e: ApiFailure.RateLimited) {
    rateLimited60 = e
}
try {
    throwApiFailureRateLimited(120u)
} catch (e: ApiFailure.RateLimited) {
    rateLimited120 = e
}

assert(rateLimited30!! < rateLimited60!!)
assert(rateLimited60!! < rateLimited120!!)
assert(rateLimited30!! < rateLimited120!!)
assert(rateLimited30!!.toString() == "api rate limited: 30s")
assert(rateLimited120!!.toString() == "api rate limited: 120s")

val apiList = listOf(apiErr3!!, apiErr1!!, apiErr2!!)
val sortedApi = apiList.sorted()
assert(sortedApi[0] is ApiFailure.NetworkIssue)
assert(sortedApi[2] is ApiFailure.ServerDown)
