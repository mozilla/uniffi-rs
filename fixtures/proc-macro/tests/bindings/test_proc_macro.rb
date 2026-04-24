# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'proc_macro'

include ProcMacro

class TestProcMacroDefaults < Test::Unit::TestCase
  def test_record_with_explicit_defaults
    r = RecordWithDefaults.new no_default_string: 'Test'

    assert_equal 'Test', r.no_default_string
    assert r.boolean
    assert_equal 42, r.integer
    assert_in_delta 4.2, r.float_var, 0.001
    assert_equal [], r.vec
    assert_nil r.opt_vec
    assert_equal 42, r.opt_integer
    assert_equal 42, r.custom_integer
    assert !r.boolean_default
    assert_equal '', r.string_default
    assert_nil r.opt_default
  end

  def test_record_with_implicit_defaults
    r = RecordWithImplicitDefaults.new

    assert !r.boolean
    assert_equal 0, r.int8
    assert_equal 0, r.uint8
    assert_equal 0, r.int16
    assert_equal 0, r.uint16
    assert_equal 0, r.int32
    assert_equal 0, r.uint32
    assert_equal 0, r.int64
    assert_equal 0, r.uint64
    assert_equal 0.0, r.afloat
    assert_equal 0.0, r.adouble
    assert_equal [], r.vec
    assert_equal({}, r.map)
    assert_equal ''.b, r.some_bytes
    assert_nil r.opt_int32
    assert_equal 0, r.custom_integer
  end

  def test_function_defaults
    assert_equal 42, ProcMacro.double_with_default
    assert_equal 1, ProcMacro.sum_with_default(1)
    assert_equal 3, ProcMacro.sum_with_default(1, 2)
  end

  def test_object_defaults
    obj = ObjectWithDefaults.new

    assert_equal 42, obj.add_to_num
    assert_equal 30, obj.add_to_implicit_num
    assert_equal 31, obj.add_to_implicit_num(1)
  end
end
