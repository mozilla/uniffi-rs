# frozen_string_literal: true

# Note that Ruby was unaffected by the bug,
# we better keep it that way.

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'regression_test_wrong_lower_check'

class TestChronological < Test::Unit::TestCase
  def test_works
    assert_equal RegressionTestWrongLowerCheck.optional_string(), nil
    assert_equal RegressionTestWrongLowerCheck.optional_string("value"), "value"
  end

  def test_klass_works
    assert_equal RegressionTestWrongLowerCheck::Klass.new.optional_string(), nil
    assert_equal RegressionTestWrongLowerCheck::Klass.new.optional_string("value"), "value"
  end

  def test_raises
     assert_raise TypeError do
       RegressionTestWrongLowerCheck::Klass.new.optional_string(1)
     end
  end
end
