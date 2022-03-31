# This is just a basic "it loaded and ran" test.
# What we're really guarding against is failure to
# load the bindings as a result of buggy codegen.

from regression_test_i356 import *

assert which(True) == Which.YEAH
assert which(False) == Which.NAH
