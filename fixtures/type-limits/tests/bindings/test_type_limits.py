# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_type_limits import *

def assert_raises(func, exception_cls):
    try:
        func()
    except exception_cls:
        return
    raise AssertionError("didn't raise the required exception")

# strict lower bounds
assert_raises(lambda: take_i8(-2**7 - 1), ValueError)
assert_raises(lambda: take_i16(-2**15 - 1), ValueError)
assert_raises(lambda: take_i32(-2**31 - 1), ValueError)
assert_raises(lambda: take_i64(-2**63 - 1), ValueError)
assert_raises(lambda: take_u8(-1), ValueError)
assert_raises(lambda: take_u16(-1), ValueError)
assert_raises(lambda: take_u32(-1), ValueError)
assert_raises(lambda: take_u64(-1), ValueError)

assert take_i8(-2**7) == -2**7
assert take_i16(-2**15) == -2**15
assert take_i32(-2**31) == -2**31
assert take_i64(-2**63) == -2**63
assert take_u8(0) == 0
assert take_u16(0) == 0
assert take_u32(0) == 0
assert take_u64(0) == 0

# strict upper bounds
assert_raises(lambda: take_i8(2**7), ValueError)
assert_raises(lambda: take_i16(2**15), ValueError)
assert_raises(lambda: take_i32(2**31), ValueError)
assert_raises(lambda: take_i64(2**63), ValueError)
assert_raises(lambda: take_u8(2**8), ValueError)
assert_raises(lambda: take_u16(2**16), ValueError)
assert_raises(lambda: take_u32(2**32), ValueError)
assert_raises(lambda: take_u64(2**64), ValueError)

assert take_i8(2**7 - 1) == 2**7 - 1
assert take_i16(2**15 - 1) == 2**15 - 1
assert take_i32(2**31 - 1) == 2**31 - 1
assert take_i64(2**63 - 1) == 2**63 - 1
assert take_u8(2**8 - 1) == 2**8 - 1
assert take_u16(2**16 - 1) == 2**16 - 1
assert take_u32(2**32 - 1) == 2**32 - 1
assert take_u64(2**64 - 1) == 2**64 - 1

# larger numbers
assert_raises(lambda: take_i8(10**3), ValueError)
assert_raises(lambda: take_i16(10**5), ValueError)
assert_raises(lambda: take_i32(10**10), ValueError)
assert_raises(lambda: take_i64(10**19), ValueError)
assert_raises(lambda: take_u8(10**3), ValueError)
assert_raises(lambda: take_u16(10**5), ValueError)
assert_raises(lambda: take_u32(10**10), ValueError)
assert_raises(lambda: take_u64(10**20), ValueError)

assert take_i8(10**2) == 10**2
assert take_i16(10**4) == 10**4
assert take_i32(10**9) == 10**9
assert take_i64(10**18) == 10**18
assert take_u8(10**2) == 10**2
assert take_u16(10**4) == 10**4
assert take_u32(10**9) == 10**9
assert take_u64(10**19) == 10**19
