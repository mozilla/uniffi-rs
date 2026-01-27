import regression_test_swift_dictionary_nesting

assert(getDict1() == [String: String]())
assert(getDict2() == [String: [String]]())
assert(getDict3() == [String: [String: String]]())
assert(getDict4() == [String: [String: [String]]]())
