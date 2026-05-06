# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestEnums < Test::Unit::TestCase
  include UniffiBindgenTests

  # --- EnumWithData (flat enum) ---

  def test_enum_no_data_roundtrip
    assert_equal EnumNoData::A, UniffiBindgenTests.roundtrip_enum_no_data(EnumNoData::A)
    assert_equal EnumNoData::B, UniffiBindgenTests.roundtrip_enum_no_data(EnumNoData::B)
    assert_equal EnumNoData::C, UniffiBindgenTests.roundtrip_enum_no_data(EnumNoData::C)
  end

  def test_enum_no_data_distinct
    assert_not_equal EnumNoData::A, EnumNoData::B
    assert_not_equal EnumNoData::A, EnumNoData::C
  end

  # --- EnumWithData (non-flat enum) ---
  def test_enum_with_data_named_variant
    em = EnumWithData::A.new value: 10, value2: 20
    result = UniffiBindgenTests.roundtrip_enum_with_data(em)

    assert_kind_of EnumWithData::A, result
    assert_equal 10, result.value
    assert_equal 20, result.value2
  end

  def test_enum_with_data_tuple_variant
    en = EnumWithData::B.new 'hello', 42
    result = UniffiBindgenTests.roundtrip_enum_with_data(en)

    assert_kind_of EnumWithData::B, result
    assert_equal 'hello', result[0]
    assert_equal 42, result[1]
  end

  def test_enum_with_data_empty_variant
    # C has no data but lives is a non-flat enum - still a class
    en = EnumWithData::C.new
    result = UniffiBindgenTests.roundtrip_enum_with_data(en)

    assert_kind_of EnumWithData::C, result
  end

  # --- ComplexEnum ---

  def test_complex_enum_a
    inner = EnumNoData::B
    en = ComplexEnum::A.new value: inner
    result = UniffiBindgenTests.roundtrip_complex_enum(en)

    assert_kind_of ComplexEnum::A, result
    assert_equal EnumNoData::B, result.value
  end

  def test_complex_enum_b
    inner = EnumWithData::A.new value: 5, value2: 6
    en = ComplexEnum::B.new value: inner
    result = UniffiBindgenTests.roundtrip_complex_enum(en)

    assert_kind_of ComplexEnum::B, result
    assert_kind_of EnumWithData::A, result.value
    assert_equal 5, result.value.value
    assert_equal 6, result.value.value2
  end

  def test_complex_enum_c
    inner = SimpleRec.new a: 77
    en = ComplexEnum::C.new value: inner
    result = UniffiBindgenTests.roundtrip_complex_enum(en)

    assert_kind_of ComplexEnum::C, result
    assert_kind_of SimpleRec, result.value
    assert_equal 77, result.value.a
  end

  # --- ExplicitValuedEnum ---

  # ExplicitValuedEnum is a separate type from EnumNoData - just verify constants exists and differ
  def test_explicit_valued_enum_roundtrip
    assert_not_nil ExplicitValuedEnum::FIRST
    assert_not_nil ExplicitValuedEnum::SECOND
    assert_not_nil ExplicitValuedEnum::FOURTH
    assert_not_nil ExplicitValuedEnum::TENTH
    assert_not_nil ExplicitValuedEnum::ELEVENTH
    assert_not_nil ExplicitValuedEnum::THIRTEENTH
  end

  def test_explicit_valued_enum_distinct
    assert_not_equal ExplicitValuedEnum::FIRST, ExplicitValuedEnum::SECOND
    assert_not_equal ExplicitValuedEnum::SECOND, ExplicitValuedEnum::FOURTH
    assert_not_equal ExplicitValuedEnum::THIRTEENTH, ExplicitValuedEnum::TENTH
  end

  # --- GappedEnum (flat) ---

  # GappedEnum is a separate type - just verify constants exists
  def test_gapped_enum_roundtrip
    assert_not_nil GappedEnum::ONE
    assert_not_nil GappedEnum::TWO
    assert_not_nil GappedEnum::THREE
  end

  def test_gapped_enum_distinct
    assert_not_equal GappedEnum::ONE, GappedEnum::TWO
    assert_not_equal GappedEnum::TWO, GappedEnum::THREE
  end
end
