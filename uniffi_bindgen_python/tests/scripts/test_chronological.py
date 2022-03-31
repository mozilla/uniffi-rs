from chronological import *
from datetime import datetime, timedelta, timezone, MAXYEAR
import sys

# Test passing timestamp and duration while returning timestamp
assert add(datetime.fromtimestamp(100.000001, timezone.utc), timedelta(seconds=1, microseconds=1)).timestamp() == 101.000002

# Test passing timestamp while returning duration
assert diff(datetime.fromtimestamp(101.000002, timezone.utc), datetime.fromtimestamp(100.000001, timezone.utc)) == timedelta(seconds=1, microseconds=1)

# Test pre-epoch timestamps
assert add(datetime.fromisoformat('1955-11-05T00:06:00.283001+00:00'), timedelta(seconds=1, microseconds=1)) == datetime.fromisoformat('1955-11-05T00:06:01.283002+00:00')

# Test exceptions are propagated
try:
    diff(datetime.fromtimestamp(100, timezone.utc), datetime.fromtimestamp(101, timezone.utc))
    assert(not("Should have thrown a TimeDiffError exception!"))
except ChronologicalError.TimeDiffError:
    # It's okay!
    pass

# Test near max timestamp bound, no microseconds due to python floating point precision issues
assert add(datetime(MAXYEAR, 12, 31, 23, 59, 59, 0, tzinfo=timezone.utc), timedelta(seconds=0)) == datetime(MAXYEAR, 12, 31, 23, 59, 59, 0, tzinfo=timezone.utc)

# Test overflow at max timestamp bound
try:
    add(datetime(MAXYEAR, 12, 31, 23, 59, 59, 0, tzinfo=timezone.utc), timedelta(seconds=1))
    assert(not("Should have thrown a ValueError exception!"))
except OverflowError:
    # It's okay!
    pass

# Test that rust timestamps behave like kotlin timestamps
pythonBefore = datetime.now(timezone.utc)
rustNow = now()
pythonAfter = datetime.now(timezone.utc)

assert pythonBefore <= rustNow <= pythonAfter

# Test that uniffi returns UTC times
assert now().tzinfo is timezone.utc
assert abs(datetime.now(timezone.utc) - now()) <= timedelta(seconds=1)

# Test that optionals work.
assert(optional(now(), timedelta(seconds=0)))
assert(not optional(None, timedelta(seconds=0)))
assert(not optional(now(), None))
