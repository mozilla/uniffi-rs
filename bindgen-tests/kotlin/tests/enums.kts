import uniffi.uniffi_bindgen_tests.*

// simple enums
assert(roundtripEnumWithData(EnumWithData.A(10.toUByte())) == EnumWithData.A(10.toUByte()))
assert(roundtripEnumWithData(EnumWithData.B("Ten")) == EnumWithData.B("Ten"))
assert(roundtripEnumWithData(EnumWithData.C) == EnumWithData.C)

// complex enums
assert(
    roundtripComplexEnum(ComplexEnum.A(EnumNoData.C)) ==
    ComplexEnum.A(EnumNoData.C))
assert(
    roundtripComplexEnum(ComplexEnum.B(EnumWithData.A(20.toUByte()))) ==
    ComplexEnum.B(EnumWithData.A(20.toUByte())))
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
