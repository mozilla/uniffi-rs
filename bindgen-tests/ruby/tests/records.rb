# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestRecords < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_simple_rec
    rec = SimpleRec.new a: 42
    result = UniffiBindgenTests.roundtrip_simple_rec rec

    assert_equal 42, result.a
  end

  def test_unit_rec
    assert_equal UnitRec.new, UnitRec.new
  end

  def test_complex_rec
    rec = ComplexRec.new(
      field_u8: 1,
      field_i8: -1,
      field_u16: 1000,
      field_i16: -1000,
      field_u32: 100_000,
      field_i32: -100_000,
      field_u64: 10_000_000_000,
      field_i64: -10_000_000_000,
      field_f32: 1.5,
      field_f64: 3.141_592_653_589_793,
      field_string: 'hello world',
      field_rec: SimpleRec.new(a: 99)
    )

    result = UniffiBindgenTests.roundtrip_complex_rec rec

    assert_equal 1, result.field_u8
    assert_equal(-1, result.field_i8)
    assert_equal 1000, result.field_u16
    assert_equal(-1000, result.field_i16)
    assert_equal 100_000, result.field_u32
    assert_equal(-100_000, result.field_i32)
    assert_equal 10_000_000_000, result.field_u64
    assert_equal(-10_000_000_000, result.field_i64)
    assert_in_delta 1.5, result.field_f32, 0.001
    assert_in_delta 3.141_592_653_589_793, result.field_f64, 0.000_000_001
    assert_equal 'hello world', result.field_string
    assert_equal 99, result.field_rec.a
  end
end
