# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestRecords < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_simple_rec
    rec = SimpleRec.new a: 42
    result = UniffiBindgenTests.roundtrip_simple_rec rec

    assert_equal 42, result.a
  end

  def test_value_ref
    result = UniffiBindgenTests.roundtrip_u8_ref 2
    assert_equal 2, result
  end

  def test_interface_ref
    interface = ReferenceTestInterface.new
    double_result = interface.double_value 2
    assert_equal 4, double_result

    double_result2 = UniffiBindgenTests.call_double_value interface, 3
    assert_equal 6, double_result2
  end

  def test_trait_interface_ref
    trait_interface = UniffiBindgenTests.create_reference_test_trait_interface
    triple_result = UniffiBindgenTests.call_triple_value_trait_interface trait_interface, 10
    assert_equal 30, triple_result
  end
end
