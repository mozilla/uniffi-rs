// This is just a basic "it compiled and run" test.
// What we're really guarding against is failure to
// compile the bindings as a result of buggy codegen.

import regression_test_i356

assert(which(arg: true) == .yeah)
assert(which(arg: false) == .nah)
