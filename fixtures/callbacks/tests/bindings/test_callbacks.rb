# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'fixture_callbacks'

class SomeOtherError < StandardError
end

class RubyGetters
  def get_bool(v, argument_two)
    v ^ argument_two
  end

  def get_string(v, arg2)
    raise FixtureCallbacks::SimpleError::BadArgument if v == 'bad-argument'
    raise SomeOtherError, 'unexpected value' if v == 'unexpected-error'

    arg2 ? '1234567890123' : v
  end

  def get_option(v, arg2)
    raise FixtureCallbacks::ComplexError::ReallyBadArgument.new(code: 20) if v == 'bad-argument'
    raise SomeOtherError, 'unexpected value' if v == 'unexpected-error'

    arg2 ? v&.upcase : v
  end

  def get_list(v, arg2)
    arg2 ? v : []
  end

  def get_nothing(v)
    raise FixtureCallbacks::SimpleError::BadArgument if v == 'bad-argument'
    raise SomeOtherError, 'unexpected value' if v == 'unexpected-error'
  end
end

class StoredRubyStringifier
  def from_simple_type(value)
    "ruby: #{value}"
  end

  # Included to ensure callback argument type collection handles nested optional sequence types.
  def from_complex_type(values)
    "ruby: #{values}"
  end
end

class TestCallbacks < Test::Unit::TestCase
  def setup
    @rust_getters = FixtureCallbacks::RustGetters.new
    @callback = RubyGetters.new
  end

  def test_roundtrip_callback_values
    [true, false].each do |v|
      flag = true
      assert_equal @callback.get_bool(v, flag), @rust_getters.get_bool(@callback, v, flag)
    end

    [[1, 2], [0, 1]].each do |v|
      flag = true
      assert_equal @callback.get_list(v, flag), @rust_getters.get_list(@callback, v, flag)
    end

    %w[Hello world].each do |v|
      flag = true
      assert_equal @callback.get_string(v, flag), @rust_getters.get_string(@callback, v, flag)
    end

    ['Some', nil].each do |v|
      flag = false
      assert_equal @callback.get_option(v, flag), @rust_getters.get_option(@callback, v, flag)
    end
  end

  def test_optional_callback_argument
    assert_equal 'TestString', @rust_getters.get_string_optional_callback(@callback, 'TestString', false)
    assert_nil @rust_getters.get_string_optional_callback(nil, 'TestString', false)
  end

  def test_void_callback_method
    assert_nothing_raised do
      @rust_getters.get_nothing(@callback, 'TestString')
    end
  end

  def test_callback_error_mapping
    assert_raises(FixtureCallbacks::SimpleError::BadArgument) do
      @rust_getters.get_string(@callback, 'bad-argument', true)
    end

    assert_raises(FixtureCallbacks::SimpleError::UnexpectedError) do
      @rust_getters.get_string(@callback, 'unexpected-error', true)
    end

    e = assert_raises(FixtureCallbacks::ComplexError::ReallyBadArgument) do
      @rust_getters.get_option(@callback, 'bad-argument', true)
    end
    assert_equal 20, e.code

    e = assert_raises(FixtureCallbacks::ComplexError::UnexpectedErrorWithReason) do
      @rust_getters.get_option(@callback, 'unexpected-error', true)
    end
    assert_kind_of String, e.reason
    assert_match e.reason, SomeOtherError.new('unexpected value').inspect

    assert_raises(FixtureCallbacks::SimpleError::BadArgument) do
      @rust_getters.get_nothing(@callback, 'bad-argument')
    end

    assert_raises(FixtureCallbacks::SimpleError::UnexpectedError) do
      @rust_getters.get_nothing(@callback, 'unexpected-error')
    end
  end

  def test_stored_callback_roundtrip
    ruby_stringifier = StoredRubyStringifier.new
    rust_stringifier = FixtureCallbacks::RustStringifier.new(ruby_stringifier)

    [1, 2].each do |v|
      assert_equal ruby_stringifier.from_simple_type(v), rust_stringifier.from_simple_type(v)
    end
  end

  def test_stored_callback_lifetime_with_multiple_references
    stringifier = StoredRubyStringifier.new
    rust_stringifier1 = FixtureCallbacks::RustStringifier.new(stringifier)
    rust_stringifier2 = FixtureCallbacks::RustStringifier.new(stringifier)

    assert_equal 'ruby: 123', rust_stringifier2.from_simple_type(123)
    nil
    GC.start

    assert_equal 'ruby: 123', rust_stringifier1.from_simple_type(123)
  end
end
