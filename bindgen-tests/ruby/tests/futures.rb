# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

# TODO: translate commented python code to ruby
class TestFutures < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_simple_calls
    assert_equal 42, UniffiBindgenTests.async_roundtrip_u8(42)
    assert_equal -42, UniffiBindgenTests.async_roundtrip_i8(-42)
    assert_equal 42, UniffiBindgenTests.async_roundtrip_u16(42)
    assert_equal -42, UniffiBindgenTests.async_roundtrip_i16(-42)
    assert_equal 42, UniffiBindgenTests.async_roundtrip_u32(42)
    assert_equal -42, UniffiBindgenTests.async_roundtrip_i32(-42)
    assert_equal 42, UniffiBindgenTests.async_roundtrip_u64(42)
    assert_equal -42, UniffiBindgenTests.async_roundtrip_i64(-42)
    assert_equal 0.5, UniffiBindgenTests.async_roundtrip_f32(0.5)
    assert_equal -0.5, UniffiBindgenTests.async_roundtrip_f64(-0.5)
    assert_equal 'hi', UniffiBindgenTests.async_roundtrip_string('hi')
    assert_equal [42], UniffiBindgenTests.async_roundtrip_vec([42])
    assert_equal(
      { 'hello' => 'world' }, 
      UniffiBindgenTests.async_roundtrip_map({ 'hello' => 'world' })
    )
  end

  def test_errors
    assert_raises(TestError::Failure1) { UniffiBindgenTests.async_throw_error }
  end

  def test_methods
    # obj = AsyncInterface.new('Alice')
    # assert_equal obj.name, 'Alice'
    #
    # obj2 = UniffiBindgenTests.async_roundtrip_obj(obj)
    # assert_equal obj2.name, 'Alice'
  end
end
