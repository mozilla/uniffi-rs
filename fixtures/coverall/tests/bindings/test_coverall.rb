# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'coverall'

class TestCoverall < Test::Unit::TestCase
  def test_some_dict
    d = Coverall.create_some_dict
    assert_equal(d.text, 'text')
    assert_equal(d.maybe_text, 'maybe_text')
    assert_true(d.a_bool)
    assert_false(d.maybe_a_bool)
    assert_equal(d.unsigned8, 1)
    assert_equal(d.maybe_unsigned8, 2)
    assert_equal(d.unsigned64, 18_446_744_073_709_551_615)
    assert_equal(d.maybe_unsigned64, 0)
    assert_equal(d.signed8, 8)
    assert_equal(d.maybe_signed8, 0)
    assert_equal(d.signed64, 9_223_372_036_854_775_807)
    assert_equal(d.maybe_signed64, 0)

    assert_in_delta(d.float32, 1.2345)
    assert_in_delta(d.maybe_float32, 22.0 / 7.0)

    assert_equal(d.float64, 0.0)
    assert_equal(d.maybe_float64, 1.0)
  end

  def test_none_dict
    d = Coverall.create_none_dict
    assert_equal(d.text, 'text')
    assert_nil(d.maybe_text)
    assert_true(d.a_bool)
    assert_nil(d.maybe_a_bool)
    assert_equal(d.unsigned8, 1)
    assert_nil(d.maybe_unsigned8)
    assert_equal(d.unsigned64, 18_446_744_073_709_551_615)
    assert_nil(d.maybe_unsigned64)
    assert_equal(d.signed8, 8)
    assert_nil(d.maybe_signed8)
    assert_equal(d.signed64, 9_223_372_036_854_775_807)
    assert_nil(d.maybe_signed64)

    assert_in_delta(d.float32, 1.2345)
    assert_nil(d.maybe_float32)
    assert_equal(d.float64, 0.0)
    assert_nil(d.maybe_float64)
  end

  def test_constructors
    assert_equal(Coverall.get_num_alive, 0)
    # must work.
    coveralls = Coverall::Coveralls.new 'c1'
    assert_equal(Coverall.get_num_alive, 1)
    # make sure it really is our Coveralls object.
    assert_equal(coveralls.get_name, 'c1')
    # must also work.
    coveralls2 = Coverall::Coveralls.fallible_new('c2', false)
    assert_equal(Coverall.get_num_alive, 2)
    # make sure it really is our Coveralls object.
    assert_equal(coveralls2.get_name, 'c2')

    assert_raise Coverall::CoverallError::TooManyHoles do
      Coverall::Coveralls.fallible_new('', true)
    end

    assert_raise Coverall::InternalError do
      Coverall::Coveralls.panicing_new('expected panic: woe is me')
    end

    assert_raise_message /expected panic: woe is me/ do
      Coverall::Coveralls.panicing_new('expected panic: woe is me')
    end

    begin
      obejcts = 10.times.map { Coverall::Coveralls.new 'c1' }
      assert_equal 12, Coverall.get_num_alive
      obejcts = nil
      GC.start
    end

    assert_equal 2, Coverall.get_num_alive
  end

  def test_errors
    coveralls = Coverall::Coveralls.new 'test_errors'
    assert_equal coveralls.get_name, 'test_errors'

    assert_raise Coverall::CoverallError::TooManyHoles do
      coveralls.maybe_throw true
    end

    assert_raise Coverall::InternalError, 'expected panic: oh no' do
      coveralls.panic 'expected panic: oh no'
    end

    assert_raise_message /expected panic: oh no/ do
      coveralls.panic 'expected panic: oh no'
    end
  end

  def test_self_by_arc
    coveralls = Coverall::Coveralls.new 'test_self_by_arc'

    # One reference is held by the handlemap, and one by the `Arc<Self>` method receiver.
    assert_equal coveralls.strong_count, 2
  end
end
