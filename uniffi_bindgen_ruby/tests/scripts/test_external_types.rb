# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'external_types_lib'

class TestExternalTypes < Test::Unit::TestCase
  def test_round_trip
    ct = ExternalTypesLib.get_combined_type(ExternalTypesLib::CombinedType.new(
      ExternalTypesLib::CrateOneType.new("test"),
      ExternalTypesLib::CrateTwoType.new(42),
    ))
    assert_equal(ct.cot.sval, "test")
    assert_equal(ct.ctt.ival, 42)
  end

  def test_none_value
    ct = ExternalTypesLib.get_combined_type(nil)
    assert_equal(ct.cot.sval, "hello")
    assert_equal(ct.ctt.ival, 1)
  end
end
