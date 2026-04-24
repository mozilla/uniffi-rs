# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'uri'
require 'custom_types'

class TestCustomTypes < Test::Unit::TestCase
  def test_get_custom_types_demo_defaults
    demo = CustomTypes.get_custom_types_demo(nil)

    assert_kind_of URI, demo.url
    assert_equal 'http://example.com/', demo.url.to_s
    assert_equal 123, demo.handle
    assert_equal 456_000, demo.time_interval_ms
    assert_equal 456.0, demo.time_interval_sec_dbl, 0.001
    assert_equal 777.0, demo.time_interval_sec_flt, 0.001
  end

  def test_roundtrip
    val = CustomTypes.get_custom_types_demo nil
    val2 = CustomTypes.get_custom_types_demo val

    assert_equal val.url, val2.url
    assert_equal val.handle, val2.handle
    assert_equal val.time_interval_ms, val2.time_interval_ms
  end

  def test_roundtrip_with_modified_url
    val = CustomTypes.get_custom_types_demo nil

    modified = CustomTypes::CustomTypesDemo.new(
      url: URI.parse('https://mozilla.org/'),
      handle: 465,
      time_interval_ms: val.time_interval_ms,
      time_interval_sec_dbl: val.time_interval_sec_dbl,
      time_interval_sec_flt: val.time_interval_sec_flt
    )

    result = CustomTypes.get_custom_types_demo modified

    assert_kind_of URI, result.url
    assert_equal 'https://mozilla.org/', result.url.to_s
    assert_equal 465, result.handle
  end

  def test_type_validation_wrong_type
    val = CustomTypes.get_custom_types_demo nil

    wrong = CustomTypes::CustomTypesDemo.new(
      url: "not a URI",
      handle: val.handle,
      time_interval_ms: val.time_interval_ms,
      time_interval_sec_dbl: val.time_interval_sec_dbl,
      time_interval_sec_flt: val.time_interval_sec_flt
    )

    assert_raise TypeError do
      CustomTypes.get_custom_types_demo wrong
    end
  end
end
