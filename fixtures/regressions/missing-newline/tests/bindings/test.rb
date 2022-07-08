# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'regression_test_missing_newline'

class TestChronological < Test::Unit::TestCase
  def test_works
    # does not throw
    RegressionTestMissingNewline.empty_func()

    assert_equal RegressionTestMissingNewline.get_dict(), {}
  end
end
