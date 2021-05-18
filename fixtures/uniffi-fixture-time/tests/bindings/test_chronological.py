from chronological import *
from datetime import datetime, timedelta, timezone, MAXYEAR
import sys

# Test passing timestamp and duration while returning timestamp
assert add(datetime.fromtimestamp(100.000001), timedelta(seconds=1, microseconds=1)).timestamp() == 101.000002

# Test passing timestamp while returning duration
assert diff(datetime.fromtimestamp(101.000002), datetime.fromtimestamp(100.000001)) == timedelta(seconds=1, microseconds=1)

# Test exceptions are propagated
try:
    diff(datetime.fromtimestamp(100), datetime.fromtimestamp(101))
    assert(not("Should have thrown a TimeDiffError exception!"))
except ChronologicalError.TimeDiffError:
    # It's okay!
    pass

# Test unix epoch lower bound
try:
    diff(datetime.fromtimestamp(-1), datetime.fromtimestamp(101))
    assert(not("Should have thrown a ValueError exception!"))
except ValueError:
    # It's okay!
    pass

# Test near max timestamp bound, no microseconds due to python floating point precision issues
assert add(datetime(MAXYEAR, 12, 31, 23, 59, 59, 0), timedelta(seconds=0)) == datetime(MAXYEAR, 12, 31, 23, 59, 59, 0)

# Test overflow at max timestamp bound
try:
    add(datetime(MAXYEAR, 12, 31, 23, 59, 59, 0), timedelta(seconds=1))
    assert(not("Should have thrown a ValueError exception!"))
except ValueError:
    # It's okay!
    pass

# Test that rust timestamps behave like kotlin timestamps
pythonBefore = datetime.now()
rustNow = now()
pythonAfter = datetime.now()

assert pythonBefore <= rustNow <= pythonAfter

# Test that uniffi returns naive datetime
assert now().tzinfo is None
