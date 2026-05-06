# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestErrors < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_failure1
    err = assert_raises(TestError::Failure1) { UniffiBindgenTests.func_with_error(0) }

    assert_kind_of TestError::Failure1, err
  end

  def test_failure2
    err = assert_raises(TestError::Failure2) { UniffiBindgenTests.func_with_error(1) }

    assert_kind_of TestError::Failure2, err
    assert_equal 'DATA', err.data
  end

  def test_failure3
    err = assert_raises(TestError::Failure3) { UniffiBindgenTests.func_with_error(50) }

    assert_kind_of TestError::Failure3, err
    assert_equal 50, err[0]
  end

  def test_flat_error
    assert_raises(TestFlatError::IoError) { UniffiBindgenTests.func_with_flat_error(0) }
  end

  # Should not raise
  def test_success
    UniffiBindgenTests.func_with_error 100
    UniffiBindgenTests.func_with_flat_error 1
  end

  # All variants are subclasses of StandardError
  def test_error_hierarchy
    assert TestError::Failure1 < StandardError
    assert TestError::Failure2 < StandardError
    assert TestError::Failure3 < StandardError
    assert TestFlatError::IoError < StandardError
  end

  def test_rescue_by_base_class
    rescued = false

    begin
      UniffiBindgenTests.func_with_error(0)
    rescue StandardError
      rescued = true
    end

    assert rescued
  end
end
