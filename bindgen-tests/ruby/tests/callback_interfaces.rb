# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class CallbackImpl
  attr_accessor :value

  @@callback_ref_count = 0

  def self.callback_ref_count
    @@callback_ref_count
  end

  def self.reset_callback_ref_count
    @@callback_ref_count = 0
  end

  def self.finalizer
    proc { @@callback_ref_count -= 1 }
  end

  def initialize(value)
    @value = value
    @@callback_ref_count += 1
    ObjectSpace.define_finalizer(self, self.class.finalizer)
  end

  def noop; end

  def get_value
    value
  end

  def set_value(value)
    self.value = value
  end

  def throw_if_equal(numbers)
    if numbers.a == 6 && numbers.b == 7
      raise 'unexpected failure'
    elsif numbers.a == numbers.b
      raise UniffiBindgenTests::TestError::Failure1
    end

    numbers
  end

  def echo(s)
    s
  end
end

# Calls that re-enter Rust from inside a callback, so several calls are in progress on the same
# thread at once. An inner call's failure must not become the result of the call it's nested in,
# and the outer call's own result must still come through after the inner calls.
class ReentrantCallback
  def reenter(depth)
    # A nested call that fails, caught here. The call this is nested in must still succeed.
    begin
      UniffiBindgenTests.func_with_error(0)
      raise 'Expected TestError::Failure1 to be raised'
    rescue UniffiBindgenTests::TestError::Failure1
      # Expected
    end
    return 0 if depth.zero?

    UniffiBindgenTests.invoke_test_reentrant_callback(self, depth - 1, false)
  end
end

class TestCallbackInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_callback_interface
    # Construct a callback interface to pass to rust
    CallbackImpl.reset_callback_ref_count
    cbi = CallbackImpl.new 42

    # Test calling callback interface methods, which we can only do indirectly.
    # Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
    UniffiBindgenTests.invoke_test_callback_interface_noop cbi
    assert_equal 42, UniffiBindgenTests.invoke_test_callback_interface_get_value(cbi)
    UniffiBindgenTests.invoke_test_callback_interface_set_value cbi, 43
    assert_equal 43, UniffiBindgenTests.invoke_test_callback_interface_get_value(cbi)
    assert_equal 'test-string', UniffiBindgenTests.invoke_test_callback_interface_echo(cbi, 'test-string')

    # The previous calls created a bunch of callback interface references.  Make sure they've been cleaned
    # up and the only remaining reference is for our `cbi` variable.
    assert_equal 1, CallbackImpl.callback_ref_count
  end

  def test_echo
    cbi = CallbackImpl.new 0

    assert_equal 'test-string', UniffiBindgenTests.invoke_test_callback_interface_echo(cbi, 'test-string')
    assert_equal '', UniffiBindgenTests.invoke_test_callback_interface_echo(cbi, '')
    assert_equal '*' * 1000, UniffiBindgenTests.invoke_test_callback_interface_echo(cbi, '*' * 1000)
  end

  def test_throw
    cbi = CallbackImpl.new 0

    result = UniffiBindgenTests.invoke_test_callback_interface_throw_if_equal(
      cbi, CallbackInterfaceNumbers.new(a: 1, b: 2)
    )

    assert_equal 1, result.a
    assert_equal 2, result.b
  end

  def test_raises_unexpected_error
    cbi = CallbackImpl.new 0
    numbers = CallbackInterfaceNumbers.new a: 1, b: 1

    assert_raises(TestError::Failure1) do
      UniffiBindgenTests.invoke_test_callback_interface_throw_if_equal cbi, numbers
    end
  end

  def test_raises_unexpected_error_converted
    cbi = CallbackImpl.new 0
    numbers = CallbackInterfaceNumbers.new a: 6, b: 7

    err = assert_raises(TestError::Failure2) do
      UniffiBindgenTests.invoke_test_callback_interface_throw_if_equal cbi, numbers
    end

    assert_kind_of String, err.data
    assert_match(/unexpected failure/, err.data)
  end

  def test_callback_interface_calls_and_errors
    CallbackImpl.reset_callback_ref_count
    cbi = CallbackImpl.new(42)

    # Only our local Ruby reference should remain alive.
    assert_equal 1, CallbackImpl.callback_ref_count
  end

  # Callback interfaces can only be passed into Rust, never returned from it, so the Rust side
  # calls a method on the callback interface and returns nil when it wasn't passed one.
  def test_optional_callback_interface
    assert_nil UniffiBindgenTests.invoke_optional_test_callback_interface_get_value(nil)

    cbi = CallbackImpl.new 42
    assert_equal 42, UniffiBindgenTests.invoke_optional_test_callback_interface_get_value(cbi)
  end

  def test_reentrant_calls
    reentrant = ReentrantCallback.new
    assert_equal 1, UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, 0, false)
    # 20 levels of Rust -> Ruby -> Rust nesting, each with a failed call nested inside it.
    assert_equal 21, UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, 20, false)
    # The outer call fails after the nested calls (including the failed ones) have finished, and
    # must report its own error.
    err = assert_raises(TestError::Failure3) do
      UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, 5, true)
    end
    assert_equal 5, err[0]
    # The same thread keeps working after the failures above.
    assert_equal 4, UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, 3, false)
  end

  # Several threads making reentrant calls, failing and succeeding, at the same time.
  def test_reentrant_calls_from_threads
    reentrant = ReentrantCallback.new
    threads = (0...8).map do |t|
      Thread.new do
        200.times do |i|
          depth = (t + i) % 4
          if (i % 3).zero?
            err = assert_raises(TestError::Failure3) do
              UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, depth, true)
            end
            assert_equal depth, err[0]
          else
            assert_equal depth + 1, UniffiBindgenTests.invoke_test_reentrant_callback(reentrant, depth, false)
          end
        end
      end
    end
    # Thread#join re-raises an exception from the thread here.
    threads.each(&:join)
  end
end
