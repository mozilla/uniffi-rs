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
