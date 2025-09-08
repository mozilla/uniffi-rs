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
assert((TraitEnum::S)("hello").toString() == "TraitEnum::S(\"hello\")")
assert((TraitEnum::I)(1).toString() == "TraitEnum::I(1)")
assert((UdlEnum::S)("hello").toString() == "S { s: \"hello\" }")

assert((UdlEnum::S)("hello") == (UdlEnum::S)("hello"))
assert((UdlEnum::S)("hello") == (UdlEnum::S)("other"))
assert((UdlEnum::S)("hello") < (UdlEnum::I)(0))
assert((TraitEnum::I)(1) == (TraitEnum::I)(1))
assert((TraitEnum::I)(1) == (TraitEnum::I)(2))
assert((TraitEnum::S)("hello") < (TraitEnum::I)(0))
