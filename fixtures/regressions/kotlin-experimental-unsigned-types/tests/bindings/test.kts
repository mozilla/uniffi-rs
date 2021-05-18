// This is just a basic "it compiled and ran" test.
// What we're really guarding against is failure to
// compile the bindings as a result of warnings about
// experimental features.

import uniffi.regression_test_kt_unsigned_types.*;

assert(returnsU16().toInt() == 16)
