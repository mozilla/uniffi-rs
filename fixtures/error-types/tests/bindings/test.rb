# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'error_types'

class TestErrorTypes < Test::Unit::TestCase
  def test_oops_raises_error_interface
    e = assert_raises(ErrorTypes::ErrorInterface) { ErrorTypes.oops }

    assert_equal ['because uniffi told me so', 'oops'], e.chain
    assert_equal 'because uniffi told me so', e.link(0)
    assert_nil e.link(99)
  end

  def test_error_interface_is_standard_error
    e = assert_raise_kind_of(StandardError) { ErrorTypes.oops }

    assert_kind_of ErrorTypes::ErrorInterface, e
  end

  def test_oops_nowrap_raises_error_interface
    e = assert_raises(ErrorTypes::ErrorInterface) { ErrorTypes.oops_nowrap }

    assert_equal ['because uniffi told me so', 'oops'], e.chain
    assert_equal 'because uniffi told me so', e.link(0)
  end

  def test_get_error_returns_without_raising
    e = ErrorTypes.get_error 'the error'

    assert_equal ['the error'], e.chain
    assert_equal 'the error', e.link(0)
    assert_nil e.link(99)
  end

  def test_throw_rich_raises_rich_error
    e = assert_raises(ErrorTypes::RichError) { ErrorTypes.throw_rich 'oh no' }

    assert_kind_of StandardError, e
  end

  def test_toops_raises_error_trait
    e = assert_raises(ErrorTypes::ErrorTrait) { ErrorTypes.toops }

    assert_equal 'trait-oops', e.msg
  end

  def test_test_interface_constructor
    ti = ErrorTypes::TestInterface.new

    assert_not_nil ti
  end

  def test_test_interface_fallible_new
    e = assert_raises(ErrorTypes::ErrorInterface) { ErrorTypes::TestInterface.fallible_new }

    assert_equal ['fallible_new'], e.chain
  end

  def test_test_interface_oops
    ti = ErrorTypes::TestInterface.new
    e = assert_raises(ErrorTypes::ErrorInterface) { ti.oops }

    assert_equal ['because the interface told me so', 'oops'], e.chain
  end

  def test_throw_proc_error
    e = assert_raises(ErrorTypes::ProcErrorInterface) { ErrorTypes.throw_proc_error 'eek' }

    assert_equal 'eek', e.message
  end

  def test_return_proc_error
    e = ErrorTypes.return_proc_error 'hello'

    assert_equal 'hello', e.message
  end

  def test_oops_enum_oops
    assert_raises(ErrorTypes::Error::Oops) { ErrorTypes.oops_enum 0 }
  end

  def test_oops_enum_value
    e = assert_raises(ErrorTypes::Error::Value) { ErrorTypes.oops_enum 1 }

    assert_equal 'value', e.value
  end

  def test_oops_enum_int_value
    e = assert_raises(ErrorTypes::Error::IntValue) { ErrorTypes.oops_enum 2 }

    assert_equal 2, e.value
  end

  def test_oops_enum_int_case_a
    e = assert_raises(ErrorTypes::Error::FlatInnerError) { ErrorTypes.oops_enum 3 }

    assert_kind_of ErrorTypes::FlatInner::CaseA, e.error
  end

  def test_oops_enum_int_case_b
    e = assert_raises(ErrorTypes::Error::FlatInnerError) { ErrorTypes.oops_enum 4 }

    assert_kind_of ErrorTypes::FlatInner::CaseB, e.error
  end

  def test_oops_enum_inner_error
    e = assert_raises(ErrorTypes::Error::InnerError) { ErrorTypes.oops_enum 5 }

    assert_kind_of ErrorTypes::Inner::CaseA, e.error
    assert_equal 'inner', e.error[0]
  end

  def test_oops_tuple_oops
    e = assert_raises(ErrorTypes::TupleError::Oops) { ErrorTypes.oops_tuple 0 }

    assert_equal 'oops', e[0]
  end

  def test_oops_tuple_value
    e = assert_raises(ErrorTypes::TupleError::Value) { ErrorTypes.oops_tuple 1 }

    assert_equal 1, e[0]
  end

  def test_oops_custom
    # CustomError is lowered as its builtin TupleError - identical behaviour to oops_tuple
    e = assert_raises(ErrorTypes::TupleError::Value) { ErrorTypes.oops_custom 1 }

    assert_equal 1, e[0]
  end

  def test_get_tuple_default
    t = ErrorTypes.get_tuple

    assert_kind_of ErrorTypes::TupleError::Oops, t
    assert_equal 'oops', t[0]
  end
end
