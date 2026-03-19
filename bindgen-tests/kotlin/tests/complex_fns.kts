import uniffi.uniffi_bindgen_tests.*

// default arguments
assert(funcWithDefault() == "DEFAULT")
assert(funcWithDefault("NON-DEFAULT") == "NON-DEFAULT")

val complexMethods = ComplexMethods()
assert(complexMethods.methodWithDefault() == "DEFAULT")
assert(complexMethods.methodWithDefault("NON-DEFAULT") == "NON-DEFAULT")

// Test argument name mapping
// the following calls will fail if the argument name differs
funcWithMultiWordArg(theArgument="test")
complexMethods.methodWithMultiWordArg(theArgument="test")
