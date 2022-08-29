# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'time'
require 'chronological'

class TestChronological < Test::Unit::TestCase
  UTC = '+00:00' # ruby 2.6 doesn't support 'UTC' literal yet.

  def test_passing_and_returning_timestamp
    time  = Time.at 100, 1, :microsecond, in: UTC
    delta = duration 1, 1, :microsecond

    assert_equal Chronological.add(time, delta), Time.at(101, 2, :microsecond, in: UTC)
  end

  def test_passing_and_returning_timestamp_with_nanoseconds
    time  = Time.at 100, 1, :microsecond, in: UTC
    delta = duration 1, 1001, :nanosecond

    assert_equal Chronological.add(time, delta), Time.at(101, 2001, :nanosecond, in: UTC)
  end

  def test_passing_timestamp_while_returning_duration
    from  = Time.at 100, 1, :microsecond, in: UTC
    to    = Time.at 101, 2, :microsecond, in: UTC
    delta = duration 1, 1, :microsecond

    assert_equal Chronological.diff(to, from), delta
  end

  def test_add_with_pre_epoch
    time  = Time.parse '1955-11-05T00:06:00.283001 UTC'
    delta = duration 1, 1, :microsecond

    assert_equal Time.parse('1955-11-05T00:06:01.283002 UTC'), Chronological.add(time, delta)
  end

  def test_exceptions_are_propagated
    assert_raises Chronological::ChronologicalError::TimeDiffError do
      Chronological.diff Time.at(100), Time.at(101)
    end
  end

  def test_rust_timestamps_behave_like_ruby_timestamps
    ruby_before = Time.now
    rust_now    = Chronological.now
    ruby_after  = Time.now

    assert ruby_before <= rust_now
    assert ruby_after >= rust_now
  end

  def test_that_uniffi_returns_UTC_times
    assert_equal 'UTC', Chronological.now.zone

    assert (Time.now.utc - Chronological.now).abs <= 1.0
  end

  private

  def duration(*args)
    Time.at(*args, in: UTC)
  end
end
