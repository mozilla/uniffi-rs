# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestUdl < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_udl
    assert_equal 2.0, UniffiBindgenTests.udl_sum_fields(UdlType.new(a: 0.5, b: 1.5))
    assert_equal UdlType.new(a: 0.5, b: 1.5), UniffiBindgenTests.udl_make_type(0.5, 1.5)
    assert_equal 2.0, UniffiBindgenTests.udl_remote_sum_fields(UdlRemoteType.new(a: 0.5, b: 1.5))
  end
end
