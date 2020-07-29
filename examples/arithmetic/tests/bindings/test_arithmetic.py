from arithmetic import *

assert add(2, 3, Overflow.SATURATING) == 5

assert equal(4, 4)
assert not equal(4,5)
