import uniffi.uniffi_bindgen_tests.*

assert(roundtripSimpleRec(SimpleRec(a=42.toUByte())) == SimpleRec(a=42.toUByte()))
assert(UnitRec() == UnitRec())
assert(
  roundtripRecWithUnitField(RecWithUnitField(unit=UnitRec(), a=42.toUByte()))
    == RecWithUnitField(unit=UnitRec(), a=42.toUByte())
)
assert(
  roundtripComplexRec(
    ComplexRec(
      fieldU8=0.toUByte(),
      fieldI8=(-1).toByte(),
      fieldU16=2.toUShort(),
      fieldI16=(-3).toShort(),
      fieldU32=4u,
      fieldI32=-5,
      fieldU64=6uL,
      fieldI64=-7L,
      fieldF32=8.5.toFloat(),
      fieldF64=9.5,
      fieldString="test",
      fieldRec=SimpleRec(a=42.toUByte())
    )
  ) == ComplexRec(
    fieldU8=0.toUByte(),
    fieldI8=(-1).toByte(),
    fieldU16=2.toUShort(),
    fieldI16=(-3).toShort(),
    fieldU32=4u,
    fieldI32=-5,
    fieldU64=6uL,
    fieldI64=-7L,
    fieldF32=8.5.toFloat(),
    fieldF64=9.5,
    fieldString="test",
    fieldRec=SimpleRec(a=42.toUByte())
  )
)
// methods
assert(SimpleRec(a=42.toUByte()).roundtrip() == SimpleRec(a=42.toUByte()))

// large records
val largeRec = createLargeRec()
assert(largeRec.a.fieldString == "test10")
assert(largeRec.c.fieldRec == SimpleRec(a=30.toUByte()))
assert(roundtripLargeRec(largeRec) == largeRec)
assert(largeRec.roundtrip() == largeRec)
val nestedLargeRec = createNestedLargeRec()
assert(nestedLargeRec.f == largeRec)
assert(roundtripNestedLargeRec(nestedLargeRec) == nestedLargeRec)
