# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestEnums < Test::Unit::TestCase
  include UniffiBindgenTests

  # CustomType1 has no foreign-side config: treat as raw u64
  def test_custom_type1
    assert_equal 100, UniffiBindgenTests.roundtrip_custom_type1(100)
  end

  # CustomType2 is lifted/lowered via a Ruby Hash per uniffi.toml Ruby config
  def test_custom_type2
    assert_equal({ 'value' => 200 }, UniffiBindgenTests.roundtrip_custom_type2({ 'value' => 200 }))
  end
end
