import uniffi.uniffi_bindgen_tests.*

// simple enums
assert(roundtripEnumWithData(EnumWithData.A(10.toUByte(), 20.toUShort())) == EnumWithData.A(10.toUByte(), 20.toUShort()))
assert(roundtripEnumWithData(EnumWithData.B("Ten", 10u)) == EnumWithData.B("Ten", 10u))
assert(roundtripEnumWithData(EnumWithData.C) == EnumWithData.C)

// Test field names for tuple-style enums
assert(EnumWithData.B("Ten", 10u).v1 == "Ten")
assert(EnumWithData.B("Ten", 10u).v2 == 10u)

// complex enums
assert(
    roundtripComplexEnum(ComplexEnum.A(EnumNoData.C)) ==
    ComplexEnum.A(EnumNoData.C))
assert(
    roundtripComplexEnum(ComplexEnum.B(EnumWithData.A(20.toUByte(), 40.toUShort()))) ==
    ComplexEnum.B(EnumWithData.A(20.toUByte(), 40.toUShort())))
assert(
    roundtripComplexEnum(ComplexEnum.C(SimpleRec(a=30.toUByte()))) ==
    ComplexEnum.C(SimpleRec(a=30.toUByte())))

// enum discriminants
assert(ExplicitValuedEnum.FIRST.value == 1.toUByte())
assert(ExplicitValuedEnum.SECOND.value == 2.toUByte())
assert(ExplicitValuedEnum.FOURTH.value == 4.toUByte())
assert(ExplicitValuedEnum.TENTH.value == 10.toUByte())
assert(ExplicitValuedEnum.ELEVENTH.value == 11.toUByte())
assert(ExplicitValuedEnum.THIRTEENTH.value == 13.toUByte())
// Some discriminants specified, increment by one for any unspecified variants
assert(GappedEnum.ONE.value == 10.toUByte())
assert(GappedEnum.TWO.value == 11.toUByte()) // Sequential value after ONE (10+1)
assert(GappedEnum.THREE.value == 14.toUByte()) // Explicit value again
