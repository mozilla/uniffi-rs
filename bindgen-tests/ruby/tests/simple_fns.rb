# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestSimpleFns < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_func
    assert_nothing_raised { UniffiBindgenTests.test_func }
  end

  def test_unexpected_error_func
    # Rust panics are wrapped in UniffiBindgenTests::InternalError
    assert_raise(UniffiBindgenTests::InternalError) do
      UniffiBindgenTests.test_unexpected_error_func
    end
  end
end
