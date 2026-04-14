import uniffi.uniffi_bindgen_tests.*

// Records
val r = RecWithDefault()
assert(r.n == 42.toUByte())
assert(r.v == listOf<Int>())

// Enums
val e = EnumWithDefault.OtherVariant()
assert(e == EnumWithDefault.OtherVariant(a="default"))

// default arguments
assert(funcWithDefault() == "DEFAULT")
assert(funcWithDefault("NON-DEFAULT") == "NON-DEFAULT")

val i = InterfaceWithDefaults()
assert(i.methodWithDefault() == "DEFAULT")
assert(i.methodWithDefault("NON-DEFAULT") == "NON-DEFAULT")
