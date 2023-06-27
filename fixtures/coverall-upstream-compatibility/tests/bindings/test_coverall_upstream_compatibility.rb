# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'coverall_upstream_compatibility'

class TestCoverall < Test::Unit::TestCase

  def test_some_dict
    d = CoverallUpstreamCompatibility.create_some_dict
    assert_equal(d.text, 'text')
    assert_equal(d.maybe_text, 'maybe_text')
    assert_true(d.a_bool)
    assert_false(d.maybe_a_bool)
    assert_equal(d.unsigned8, 1)
    assert_equal(d.maybe_unsigned8, 2)
    assert_equal(d.unsigned16, 3)
    assert_equal(d.maybe_unsigned16, 4)
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

    assert_equal(d.coveralls.get_name(), "some_dict")
  end

  def test_none_dict
    d = CoverallUpstreamCompatibility.create_none_dict
    assert_equal(d.text, 'text')
    assert_nil(d.maybe_text)
    assert_true(d.a_bool)
    assert_nil(d.maybe_a_bool)
    assert_equal(d.unsigned8, 1)
    assert_nil(d.maybe_unsigned8)
    assert_equal(d.unsigned16, 3)
    assert_nil(d.maybe_unsigned16)
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
    GC.start
    assert_equal(CoverallUpstreamCompatibility.get_num_alive, 0)
    # must work.
    coveralls = CoverallUpstreamCompatibility::Coveralls.new 'c1'
    assert_equal(CoverallUpstreamCompatibility.get_num_alive, 1)
    # make sure it really is our Coveralls object.
    assert_equal(coveralls.get_name, 'c1')
    # must also work.
    coveralls2 = CoverallUpstreamCompatibility::Coveralls.fallible_new('c2', false)
    assert_equal(CoverallUpstreamCompatibility.get_num_alive, 2)
    # make sure it really is our Coveralls object.
    assert_equal(coveralls2.get_name, 'c2')

    assert_raise CoverallUpstreamCompatibility::CoverallError::TooManyHoles do
      CoverallUpstreamCompatibility::Coveralls.fallible_new('', true)
    end

    assert_raise CoverallUpstreamCompatibility::InternalError do
      CoverallUpstreamCompatibility::Coveralls.panicking_new('expected panic: woe is me')
    end

    assert_raise_message /expected panic: woe is me/ do
      CoverallUpstreamCompatibility::Coveralls.panicking_new('expected panic: woe is me')
    end

    begin
      objects = 10.times.map { CoverallUpstreamCompatibility::Coveralls.new 'c1' }
      assert_equal 12, CoverallUpstreamCompatibility.get_num_alive
      objects = nil
      GC.start
    end

    assert_equal 2, CoverallUpstreamCompatibility.get_num_alive
  end

  def test_simple_errors
    coveralls = CoverallUpstreamCompatibility::Coveralls.new 'test_simple_errors'
    assert_equal coveralls.get_name, 'test_simple_errors'

    err = assert_raise CoverallUpstreamCompatibility::CoverallError::TooManyHoles do
      coveralls.maybe_throw true
    end
    assert_equal err.message, 'The coverall has too many holes'

    assert_raise CoverallUpstreamCompatibility::CoverallError::TooManyHoles do
      coveralls.maybe_throw_into true
    end

    err = assert_raise CoverallUpstreamCompatibility::InternalError do
      coveralls.panic 'expected panic: oh no'
    end
    assert_equal err.message, 'expected panic: oh no'

    assert_raise_message /expected panic: oh no/ do
      coveralls.panic 'expected panic: oh no'
    end
  end

  def test_complex_errors
    coveralls = CoverallUpstreamCompatibility::Coveralls.new 'test_complex_errors'
    assert_equal coveralls.maybe_throw_complex(0), true

    begin
      coveralls.maybe_throw_complex(1)
    rescue CoverallUpstreamCompatibility::ComplexError::OsError => err
      assert_equal err.code, 10
      assert_equal err.extended_code, 20
      assert_equal err.to_s, 'CoverallUpstreamCompatibility::ComplexError::OsError(code=10, extended_code=20)'
    else
      raise 'should have thrown'
    end

    begin
      coveralls.maybe_throw_complex(2)
    rescue CoverallUpstreamCompatibility::ComplexError::PermissionDenied => err
      assert_equal err.reason, "Forbidden"
      assert_equal err.to_s, 'CoverallUpstreamCompatibility::ComplexError::PermissionDenied(reason="Forbidden")'
    else
      raise 'should have thrown'
    end

    assert_raise CoverallUpstreamCompatibility::InternalError do
      coveralls.maybe_throw_complex(3)
    end
  end

  def test_self_by_arc
    coveralls = CoverallUpstreamCompatibility::Coveralls.new 'test_self_by_arc'

    # One reference is held by the handlemap, and one by the `Arc<Self>` method receiver.
    assert_equal coveralls.strong_count, 2
  end

  def test_arcs
    GC.start
    coveralls = CoverallUpstreamCompatibility::Coveralls.new 'test_arcs'
    assert_equal 1, CoverallUpstreamCompatibility.get_num_alive

    assert_equal 2, coveralls.strong_count
    assert_equal nil, coveralls.get_other

    coveralls.take_other coveralls
    # should now be a new strong ref.
    assert_equal 3, coveralls.strong_count
    # but the same number of instances.
    assert_equal 1,  CoverallUpstreamCompatibility.get_num_alive
    # and check it's the correct object.
    assert_equal "test_arcs",  coveralls.get_other.get_name

    # Using `assert_raise` here would keep a reference to `coveralls` alive
    # by capturing it in a closure, which would interfere with the tests.
    begin
      coveralls.take_other_fallible
    rescue CoverallUpstreamCompatibility::CoverallError::TooManyHoles
      # OK
    else
      raise 'should have thrown'
    end

    begin
      coveralls.take_other_panic "expected panic: with an arc!"
    rescue CoverallUpstreamCompatibility::InternalError => err
      assert_match /expected panic: with an arc!/, err.message
    else
      raise 'should have thrown'
    end

    coveralls.take_other nil
    GC.start
    assert_equal 2,  coveralls.strong_count

    # Reference cleanup includes the cached most recent exception.
    coveralls = nil
    GC.start
    assert_equal 0,  CoverallUpstreamCompatibility.get_num_alive

  end

  def test_return_objects
    GC.start
    coveralls = CoverallUpstreamCompatibility::Coveralls.new "test_return_objects"
    assert_equal CoverallUpstreamCompatibility.get_num_alive, 1
    assert_equal coveralls.strong_count, 2
    c2 = coveralls.clone_me()
    assert_equal c2.get_name(), coveralls.get_name()
    assert_equal CoverallUpstreamCompatibility.get_num_alive(), 2
    assert_equal c2.strong_count(), 2

    coveralls.take_other(c2)
    # same number alive but `c2` has an additional ref count.
    assert_equal CoverallUpstreamCompatibility.get_num_alive(), 2
    assert_equal coveralls.strong_count(), 2
    assert_equal c2.strong_count(), 3

    # We can drop Ruby's reference to `c2`, but the Rust struct will not
    # be dropped as coveralls hold an `Arc<>` to it.
    c2 = nil
    GC.start
    assert_equal CoverallUpstreamCompatibility.get_num_alive(), 2

    # Dropping `coveralls` will kill both.
    coveralls = nil
    GC.start
    assert_equal CoverallUpstreamCompatibility.get_num_alive(), 0
  end

  def test_bad_objects
    coveralls = CoverallUpstreamCompatibility::Coveralls.new "test_bad_objects"
    patch = CoverallUpstreamCompatibility::Patch.new CoverallUpstreamCompatibility::Color::RED
    # `coveralls.take_other` wants `Coveralls` not `Patch`
    assert_raise_message /Expected a Coveralls instance, got.*Patch/ do
      coveralls.take_other patch
    end
  end


end
