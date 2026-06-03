# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestOptions < Test::Unit::TestCase
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

    rec = OptionsRec.new(a: 42)
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
end
