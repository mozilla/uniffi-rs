
from arithmetic import *

print("Alrighty, let's do some arithmetic FROM RUST, IN PYTHON!")
print("2 + 3 = {}".format(add(2, 3, Overflow.SATURATING)))
