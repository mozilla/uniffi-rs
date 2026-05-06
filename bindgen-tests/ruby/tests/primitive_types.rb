# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestPrimitiveTypes < Test::Unit::TestCase
  def test_input
    UniffiBindgenTests.input_u8 255
    UniffiBindgenTests.input_i8(-128)
    UniffiBindgenTests.input_u16 65_535
    UniffiBindgenTests.input_i16(-32_768)
    UniffiBindgenTests.input_u32 4_294_967_295
    UniffiBindgenTests.input_i32(-2_147_483_648)
    UniffiBindgenTests.input_u64 18_446_744_073_709_551_615
    UniffiBindgenTests.input_i64(-9_223_372_036_854_775_808)
    UniffiBindgenTests.input_f32 3.14
    UniffiBindgenTests.input_f64 3.141_592_653_589_793
    UniffiBindgenTests.input_bool true
    UniffiBindgenTests.input_string 'test-string'
  end

  def test_output
    assert_equal 1, UniffiBindgenTests.output_u8
    assert_equal 1, UniffiBindgenTests.output_i8
    assert_equal 1, UniffiBindgenTests.output_u16
    assert_equal 1, UniffiBindgenTests.output_i16
    assert_equal 1, UniffiBindgenTests.output_u32
    assert_equal 1, UniffiBindgenTests.output_i32
    assert_equal 1, UniffiBindgenTests.output_u64
    assert_equal 1, UniffiBindgenTests.output_i64
    assert_in_delta 1.0, UniffiBindgenTests.output_f32, 0.0001
    assert_in_delta 1.0, UniffiBindgenTests.output_f64, 0.0001
    assert_equal UniffiBindgenTests.output_bool, true
    assert_equal UniffiBindgenTests.output_string, 'test-string'
  end

  def test_roundtrip
    assert_equal 42, UniffiBindgenTests.roundtrip_u8(42)
    assert_equal(-42, UniffiBindgenTests.roundtrip_i8(-42))
    assert_equal 1000, UniffiBindgenTests.roundtrip_u16(1000)
    assert_equal(-1000, UniffiBindgenTests.roundtrip_i16(-1000))
    assert_equal 1_000_000, UniffiBindgenTests.roundtrip_u32(1_000_000)
    assert_equal(-1_000_000, UniffiBindgenTests.roundtrip_i32(-1_000_000))
    assert_equal 1_000_000_000, UniffiBindgenTests.roundtrip_u64(1_000_000_000)
    assert_equal(-1_000_000_000, UniffiBindgenTests.roundtrip_i64(-1_000_000_000))
    assert_in_delta 3.14, UniffiBindgenTests.roundtrip_f32(3.14), 0.0001
    assert_in_delta 3.141_592, UniffiBindgenTests.roundtrip_f64(3.141_592), 0.000_001
    assert UniffiBindgenTests.roundtrip_bool(true)
    assert !UniffiBindgenTests.roundtrip_bool(false)
    assert_equal 'test-string', UniffiBindgenTests.roundtrip_string('test-string')
  end

  def test_sum_with_many_types
    result = UniffiBindgenTests.sum_with_many_types 1, -2, 3, -4, 5, -6, 7, -8, 9.5, -10.5, true

    assert_in_delta 5.0, result, 0.0001
  end

  def test_func_with_multi_word_arg
    assert_equal 16, UniffiBindgenTests.func_with_multi_word_arg(16)
  end
end
