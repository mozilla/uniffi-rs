# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_simple_iface'

class TestSimpleIface < Test::Unit::TestCase
  def test_simple_iface
    obj = UniffiSimpleIface.make_object 9000

    assert_equal 9000, obj.get_inner()

    obj.some_method()
  end
end
