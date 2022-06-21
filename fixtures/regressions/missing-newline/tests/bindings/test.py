# This is just a basic "it loaded and ran" test.
# What we're really guarding against is failure to
# load the bindings as a result of buggy codegen.

from regression_test_missing_newline import *

# does not throw
empty_func()

assert get_dict() == {}
