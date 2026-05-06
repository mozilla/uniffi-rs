# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestCompoundTypes < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_option_some
    assert_equal 42, UniffiBindgenTests.roundtrip_option_u8(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_i8(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_u16(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_i16(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_u32(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_i32(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_u64(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_i64(42)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_f32(42.0)
    assert_equal 42, UniffiBindgenTests.roundtrip_option_f64(42.0)
    assert_equal "test-string", UniffiBindgenTests.roundtrip_option_string("test-string")
    assert_equal true, UniffiBindgenTests.roundtrip_option_bool(true)

    rec = CompoundTypesRec.new(a: 42)
    assert_equal rec, UniffiBindgenTests.roundtrip_option_rec(rec)
  end

  def test_option_none
    assert_nil UniffiBindgenTests.roundtrip_option_u8(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_i8(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_u16(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_i16(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_u32(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_i32(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_u64(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_i64(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_f32(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_f64(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_string(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_string(nil)
    assert_nil UniffiBindgenTests.roundtrip_option_rec(nil)
  end

  def test_vecs
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i8([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u16([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i16([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u32([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i32([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u64([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i64([1, 2, 3])
    assert_equal ["test-string"], UniffiBindgenTests.roundtrip_vec_string(["test-string"])
    assert_equal [true, false], UniffiBindgenTests.roundtrip_vec_bool([true, false])

    rec = CompoundTypesRec.new(a: 42)
    assert_equal [rec], UniffiBindgenTests.roundtrip_vec_rec([rec])
  end

  def test_hash_map
    map = { 'a' => 1, 'b' => 2 }

    assert_equal map, UniffiBindgenTests.roundtrip_hash_map(map)
    assert_equal({}, UniffiBindgenTests.roundtrip_hash_map({}))
  end

  def test_rec_with_compounds
    rec = RecWithCompounds.new(
      a: EnumWithCompounds::A.new(nil),
      b: nil,
      c: [true, false],
      d: { 'a' => 1, 'b' => 2 },
    )

    assert_equal rec, UniffiBindgenTests.roundtrip_rec_with_compounds(rec)
  end

  def test_complex_compound_some
    inner = [{
      'a' => CompoundTypesComplexRec.new(a: 10, b: "test", c: CompoundTypesEnum::A.new(100)),
      'b' => CompoundTypesComplexRec.new(a: 20, b: "test2", c: CompoundTypesEnum::B.new(a: 1.0, b: true)),
    }]
    assert_equal inner, UniffiBindgenTests.roundtrip_complex_compound(inner)
  end

  def test_complex_compound_none
    assert_nil UniffiBindgenTests.roundtrip_complex_compound(nil)
  end
end
