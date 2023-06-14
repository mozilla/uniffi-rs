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

    assert_raise FloatDomainError do UniffiTypeLimits.take_i8(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i16(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i32(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i64(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u8(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u16(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u32(-Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u64(-Float::INFINITY) end

    assert_raise FloatDomainError do UniffiTypeLimits.take_i8(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i16(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i32(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i64(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u8(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u16(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u32(Float::INFINITY) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u64(Float::INFINITY) end

    assert_raise FloatDomainError do UniffiTypeLimits.take_i8(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i16(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i32(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_i64(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u8(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u16(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u32(Float::NAN) end
    assert_raise FloatDomainError do UniffiTypeLimits.take_u64(Float::NAN) end
  end
  class NonInteger
  end
  def test_non_integer
    assert_raise TypeError do UniffiTypeLimits.take_i8(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_i16(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_i32(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_i64(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_u8(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_u16(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_u32(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_u64(nil) end

    assert_raise TypeError do UniffiTypeLimits.take_i8("0") end
    assert_raise TypeError do UniffiTypeLimits.take_i16("0") end
    assert_raise TypeError do UniffiTypeLimits.take_i32("0") end
    assert_raise TypeError do UniffiTypeLimits.take_i64("0") end
    assert_raise TypeError do UniffiTypeLimits.take_u8("0") end
    assert_raise TypeError do UniffiTypeLimits.take_u16("0") end
    assert_raise TypeError do UniffiTypeLimits.take_u32("0") end
    assert_raise TypeError do UniffiTypeLimits.take_u64("0") end

    assert_raise TypeError do UniffiTypeLimits.take_i8(false) end
    assert_raise TypeError do UniffiTypeLimits.take_i16(false) end
    assert_raise TypeError do UniffiTypeLimits.take_i32(false) end
    assert_raise TypeError do UniffiTypeLimits.take_i64(false) end
    assert_raise TypeError do UniffiTypeLimits.take_u8(false) end
    assert_raise TypeError do UniffiTypeLimits.take_u16(false) end
    assert_raise TypeError do UniffiTypeLimits.take_u32(false) end
    assert_raise TypeError do UniffiTypeLimits.take_u64(false) end

    assert_raise TypeError do UniffiTypeLimits.take_i8(true) end
    assert_raise TypeError do UniffiTypeLimits.take_i16(true) end
    assert_raise TypeError do UniffiTypeLimits.take_i32(true) end
    assert_raise TypeError do UniffiTypeLimits.take_i64(true) end
    assert_raise TypeError do UniffiTypeLimits.take_u8(true) end
    assert_raise TypeError do UniffiTypeLimits.take_u16(true) end
    assert_raise TypeError do UniffiTypeLimits.take_u32(true) end
    assert_raise TypeError do UniffiTypeLimits.take_u64(true) end

    assert_raise TypeError do UniffiTypeLimits.take_i8(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_i16(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_i32(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_i64(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_u8(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_u16(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_u32(NonInteger.new) end
    assert_raise TypeError do UniffiTypeLimits.take_u64(NonInteger.new) end
  end
  class IntegerLike
    def to_int
      123
    end
  end
  def test_integer_like
    assert_equal(UniffiTypeLimits.take_i8(123.0), 123)
    assert_equal(UniffiTypeLimits.take_i16(123.0), 123)
    assert_equal(UniffiTypeLimits.take_i32(123.0), 123)
    assert_equal(UniffiTypeLimits.take_i64(123.0), 123)
    assert_equal(UniffiTypeLimits.take_u8(123.0), 123)
    assert_equal(UniffiTypeLimits.take_u16(123.0), 123)
    assert_equal(UniffiTypeLimits.take_u32(123.0), 123)
    assert_equal(UniffiTypeLimits.take_u64(123.0), 123)

    assert_equal(UniffiTypeLimits.take_i8(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_i16(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_i32(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_i64(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_u8(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_u16(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_u32(-0.5), 0)
    assert_equal(UniffiTypeLimits.take_u64(-0.5), 0)

    assert_equal(UniffiTypeLimits.take_i8(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_i16(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_i32(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_i64(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_u8(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_u16(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_u32(IntegerLike.new), 123)
    assert_equal(UniffiTypeLimits.take_u64(IntegerLike.new), 123)
  end
  class NonFloat
  end
  def test_non_float
    assert_raise TypeError do UniffiTypeLimits.take_f32(nil) end
    assert_raise TypeError do UniffiTypeLimits.take_f64(nil) end

    assert_raise TypeError do UniffiTypeLimits.take_f32("0") end
    assert_raise TypeError do UniffiTypeLimits.take_f64("0") end

    assert_raise TypeError do UniffiTypeLimits.take_f32(false) end
    assert_raise TypeError do UniffiTypeLimits.take_f64(false) end

    assert_raise TypeError do UniffiTypeLimits.take_f32(true) end
    assert_raise TypeError do UniffiTypeLimits.take_f64(true) end

    assert_raise RangeError do UniffiTypeLimits.take_f32(1i) end
    assert_raise RangeError do UniffiTypeLimits.take_f64(1i) end

    assert_raise TypeError do UniffiTypeLimits.take_f32(NonFloat.new) end
    assert_raise TypeError do UniffiTypeLimits.take_f64(NonFloat.new) end
  end
  def test_float_like
    assert_equal(UniffiTypeLimits.take_f32(456), 456.0)
    assert_equal(UniffiTypeLimits.take_f64(456), 456.0)
  end
  def test_special_floats
    assert_equal(UniffiTypeLimits.take_f32(Float::INFINITY), Float::INFINITY)
    assert_equal(UniffiTypeLimits.take_f64(Float::INFINITY), Float::INFINITY)

    assert_equal(UniffiTypeLimits.take_f32(-Float::INFINITY), -Float::INFINITY)
    assert_equal(UniffiTypeLimits.take_f64(-Float::INFINITY), -Float::INFINITY)

    assert_equal(UniffiTypeLimits.take_f32(0.0).to_s, "0.0")
    assert_equal(UniffiTypeLimits.take_f64(0.0).to_s, "0.0")

    assert_equal(UniffiTypeLimits.take_f32(-0.0).to_s, "-0.0")
    assert_equal(UniffiTypeLimits.take_f64(-0.0).to_s, "-0.0")

    assert(UniffiTypeLimits.take_f32(Float::NAN).nan?)
    assert(UniffiTypeLimits.take_f64(Float::NAN).nan?)
  end
  def test_strings
    assert_raise Encoding::InvalidByteSequenceError do UniffiTypeLimits.take_string("\xff") end # invalid byte
    assert_raise Encoding::InvalidByteSequenceError do UniffiTypeLimits.take_string("\xed\xa0\x80") end # surrogate
    assert_equal(UniffiTypeLimits.take_string(""), "")
    assert_equal(UniffiTypeLimits.take_string("愛"), "愛")
    assert_equal(UniffiTypeLimits.take_string("💖"), "💖")
  end
end
