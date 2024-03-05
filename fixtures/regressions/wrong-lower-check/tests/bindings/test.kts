// Check that we can call the function and get a value back

import uniffi.regression_test_wrong_lower_check.*;

assert(optionalString() == "none")
assert(optionalString("value") == "value")
