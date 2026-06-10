# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestFutures < Test::Unit::TestCase
  include UniffiBindgenTests

  class AsyncCallbackImpl
    @@ref_count = 0

    def self.reset_ref_count
      @@ref_count = 0
    end

    def self.define_finalizer
      Proc.new { |_id| @@ref_count -= 1 }
    end

    def initialize(value)
      @value = value
      @@ref_count += 1
      ObjectSpace.define_finalizer(self, self.class.define_finalizer)
    end

    def noop
    end

    def get_value
      @value
    end

    def set_value(value)
      @value = value
    end

    def throw_if_equal(numbers)
      if numbers.a == numbers.b
        raise UniffiBindgenTests::TestError::Failure1.new
      end
      numbers
    end
  end

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
    obj = AsyncInterface.new 'Alice'
    assert_equal 'Alice', obj.name

    obj2 = UniffiBindgenTests.async_roundtrip_obj obj
    assert_equal 'Alice', obj2.name
  end

  def test_async_callback_interfaces
    cbi = AsyncCallbackImpl.new 42

    UniffiBindgenTests.invoke_test_async_callback_interface_noop cbi

    assert_equal 42, UniffiBindgenTests.invoke_test_async_callback_interface_get_value(cbi)
    UniffiBindgenTests.invoke_test_async_callback_interface_set_value cbi, 43
    assert_equal 43, UniffiBindgenTests.invoke_test_async_callback_interface_get_value(cbi)

    assert_raises(TestError::Failure1) do
      UniffiBindgenTests.invoke_test_async_callback_interface_throw_if_equal(
        cbi, 
        CallbackInterfaceNumbers.new(a: 10, b: 10)
      )
    end
    
    assert_equal(
      CallbackInterfaceNumbers.new(a: 10, b: 11), 
      UniffiBindgenTests.invoke_test_async_callback_interface_throw_if_equal(
        cbi, 
        CallbackInterfaceNumbers.new(a: 10, b: 11)
      )
    )
  end
end
