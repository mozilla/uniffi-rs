# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestDefaults < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_record_with_defaults
    rec = RecWithDefault.new

    assert_equal 42, rec.n
    assert_equal [], rec.v
  end

  def test_record_override_defaults
    rec = RecWithDefault.new(n: 123, v: [1, 2, 3])

    assert_equal 123, rec.n
    assert_equal [1, 2, 3], rec.v
  end

  def test_enum_other_variant_default_fields
    en = EnumWithDefault::OTHER_VARIANT.new

    assert_equal 'default', en.a
  end

  def test_enum_other_variant_override
    en = EnumWithDefault::OTHER_VARIANT.new(a: 'override')

    assert_equal 'override', en.a
  end

  def test_func_with_default
    assert_equal 'DEFAULT', UniffiBindgenTests.func_with_default
  end

  def test_func_with_default_override
    assert_equal 'NON-DEFAULT', UniffiBindgenTests.func_with_default('NON-DEFAULT')
  end

  def test_interface_method_with_default
    iface = InterfaceWithDefaults.new

    assert_equal 'DEFAULT', iface.method_with_default
  end

  def test_interface_method_with_default_override
    iface = InterfaceWithDefaults.new

    assert_equal 'NON-DEFAULT', iface.method_with_default('NON-DEFAULT')
  end
end
