# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

# In Ruby, both Duration and SystemTime are represented as Ruby Time objects.
# Duration must have a non-negative tv_sec.
# SystemTime maps to a UTC Time around the Unix epoch.

class TestTime < Test::Unit::TestCase
  UTC = '+00:00'

  include UniffiBindgenTests

  def test_roundtrip_duration_whole_seconds
    dur = Time.at(3600, 0, :nanosecond, in: UTC).utc

    result = UniffiBindgenTests.roundtrip_duration dur

    assert_equal dur.tv_sec, result.tv_sec
    assert_equal dur.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_duration_with_nanoseconds
    dur = Time.at(3600, 500_000_000, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_duration dur

    assert_equal dur.tv_sec, result.tv_sec
    assert_equal dur.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_duration_zero
    dur = Time.at(0, 0, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_duration dur

    assert_equal dur.tv_sec, result.tv_sec
    assert_equal dur.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_duration_negative_raises
    assert_raises(ArgumentError) do
      UniffiBindgenTests.roundtrip_duration Time.at(-1, 0, :nanosecond, in: UTC).utc
    end
  end

  def test_roundtrip_systemtime_epoch
    t = Time.at(0, 0, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_systemtime t

    assert_equal 0, result.tv_sec
    assert_equal 0, result.tv_nsec
  end

  def test_roundtrip_systemtime_post_epoch
    # 2019-01-01 00:00:00 UTC
    t = Time.at(1_546_340_800, 0, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_systemtime t

    assert_equal t.tv_sec, result.tv_sec
    assert_equal t.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_systemtime_with_nanoseconds
    t = Time.at(1_546_340_800, 123_456_789, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_systemtime t

    assert_equal t.tv_sec, result.tv_sec
    assert_equal t.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_systemtime_pre_epoch
    # 1 second before the epoch, no subsecond precision
    t = Time.at(-1, 0, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_systemtime t

    assert_equal t.tv_sec, result.tv_sec
    assert_equal t.tv_nsec, result.tv_nsec
  end

  def test_roundtrip_systemtime_pre_epoch_with_nanoseconds
    # 2 second before the epoch, with subsecond precision
    t = Time.at(-2, 500_000_000, :nanosecond, in: UTC).utc
    result = UniffiBindgenTests.roundtrip_systemtime t

    assert_equal t.tv_sec, result.tv_sec
    assert_equal t.tv_nsec, result.tv_nsec
  end
end
