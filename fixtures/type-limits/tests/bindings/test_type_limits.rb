# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'uniffi_type_limits'

class TestTypeLimits < Test::Unit::TestCase
  def test_strict_lower_bounds
    assert_raise RangeError do UniffiTypeLimits.take_i8(-2**7 - 1) end
    assert_raise RangeError do UniffiTypeLimits.take_i16(-2**15 - 1) end
    assert_raise RangeError do UniffiTypeLimits.take_i32(-2**31 - 1) end
    assert_raise RangeError do UniffiTypeLimits.take_i64(-2**63 - 1) end
    assert_raise RangeError do UniffiTypeLimits.take_u8(-1) end
    assert_raise RangeError do UniffiTypeLimits.take_u16(-1) end
    assert_raise RangeError do UniffiTypeLimits.take_u32(-1) end
    assert_raise RangeError do UniffiTypeLimits.take_u64(-1) end

    assert_equal(UniffiTypeLimits.take_i8(-2**7), -2**7)
    assert_equal(UniffiTypeLimits.take_i16(-2**15), -2**15)
    assert_equal(UniffiTypeLimits.take_i32(-2**31), -2**31)
    assert_equal(UniffiTypeLimits.take_i64(-2**63), -2**63)
    assert_equal(UniffiTypeLimits.take_u8(0), 0)
    assert_equal(UniffiTypeLimits.take_u16(0), 0)
    assert_equal(UniffiTypeLimits.take_u32(0), 0)
    assert_equal(UniffiTypeLimits.take_u64(0), 0)
  end
  def test_strict_upper_bounds
    assert_raise RangeError do UniffiTypeLimits.take_i8(2**7) end
    assert_raise RangeError do UniffiTypeLimits.take_i16(2**15) end
    assert_raise RangeError do UniffiTypeLimits.take_i32(2**31) end
    assert_raise RangeError do UniffiTypeLimits.take_i64(2**63) end
    assert_raise RangeError do UniffiTypeLimits.take_u8(2**8) end
    assert_raise RangeError do UniffiTypeLimits.take_u16(2**16) end
    assert_raise RangeError do UniffiTypeLimits.take_u32(2**32) end
    assert_raise RangeError do UniffiTypeLimits.take_u64(2**64) end

    assert_equal(UniffiTypeLimits.take_i8(2**7 - 1), 2**7 - 1)
    assert_equal(UniffiTypeLimits.take_i16(2**15 - 1), 2**15 - 1)
    assert_equal(UniffiTypeLimits.take_i32(2**31 - 1), 2**31 - 1)
    assert_equal(UniffiTypeLimits.take_i64(2**63 - 1), 2**63 - 1)
    assert_equal(UniffiTypeLimits.take_u8(2**8 - 1), 2**8 - 1)
    assert_equal(UniffiTypeLimits.take_u16(2**16 - 1), 2**16 - 1)
    assert_equal(UniffiTypeLimits.take_u32(2**32 - 1), 2**32 - 1)
    assert_equal(UniffiTypeLimits.take_u64(2**64 - 1), 2**64 - 1)
  end
  def test_larger_numbers
    assert_raise RangeError do UniffiTypeLimits.take_i8(10**3) end
    assert_raise RangeError do UniffiTypeLimits.take_i16(10**5) end
    assert_raise RangeError do UniffiTypeLimits.take_i32(10**10) end
    assert_raise RangeError do UniffiTypeLimits.take_i64(10**19) end
    assert_raise RangeError do UniffiTypeLimits.take_u8(10**3) end
    assert_raise RangeError do UniffiTypeLimits.take_u16(10**5) end
    assert_raise RangeError do UniffiTypeLimits.take_u32(10**10) end
    assert_raise RangeError do UniffiTypeLimits.take_u64(10**20) end

    assert_equal(UniffiTypeLimits.take_i8(10**2), 10**2)
    assert_equal(UniffiTypeLimits.take_i16(10**4), 10**4)
    assert_equal(UniffiTypeLimits.take_i32(10**9), 10**9)
    assert_equal(UniffiTypeLimits.take_i64(10**18), 10**18)
    assert_equal(UniffiTypeLimits.take_u8(10**2), 10**2)
    assert_equal(UniffiTypeLimits.take_u16(10**4), 10**4)
    assert_equal(UniffiTypeLimits.take_u32(10**9), 10**9)
    assert_equal(UniffiTypeLimits.take_u64(10**19), 10**19)
  end
end
