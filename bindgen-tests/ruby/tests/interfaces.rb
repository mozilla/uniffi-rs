# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestInterfaces < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_basic_interface
    iface = TestInterface.new 20

    assert_equal 20, iface.get_value
  end

  def test_clone_interface
    iface = TestInterface.new 20
    cloned = UniffiBindgenTests.clone_interface iface

    assert_equal 20, cloned.get_value
  end

  def test_optional_interface
    assert_nil UniffiBindgenTests.roundtrip_optional_interface(nil)

    iface = TestInterface.new 20
    assert_equal 20, UniffiBindgenTests.roundtrip_optional_interface(iface).get_value
  end

  def test_secondary_constructor
    iface = TestInterface.secondary_constructor 20

    assert_equal 40, iface.get_value
  end

  def test_records_with_interface_fields
    two = TwoTestInterfaces.new(
      first: TestInterface.new(1),
      second: TestInterface.new(2)
    )
    swapped = UniffiBindgenTests.swap_test_interfaces two

    assert_equal 2, swapped.first.get_value
    assert_equal 1, swapped.second.get_value
  end

  def test_enums_with_intefaces
    one = TestInterfaceEnum::ONE.new i: TestInterface.new(1)
    two = TestInterfaceEnum::TWO.new i: TestInterface.new(2)

    assert_equal 1, one.i.get_value
    assert_equal 2, two.i.get_value
  end

  def test_interface_ref_count
    iface = TestInterface.new 42

    # Single reference
    assert_equal 1, iface.ref_count

    # After clone there are 2 references: the original and the clone still alive in scope
    UniffiBindgenTests.clone_interface iface
    assert_equal 2, iface.ref_count

    # Drop the clone
    nil
    GC.start

    # After GC, back to 1 referece held by `iface`
    assert_equal 1, iface.ref_count
  end

  def test_multi_word_arg
    iface = TestInterface.new 0

    assert_equal 'test', iface.method_with_multi_word_arg('test')
  end
end
