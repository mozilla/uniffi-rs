# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uri'
require 'uniffi_bindgen_tests'

class TestExternalTypes < Test::Unit::TestCase
  Ext = UniffiBindgenTestsExternalTypesSource
  Mid = UniffiBindgenTestsMidTypes

  def test_ext_record
    rec = Ext::ExternalRec.new(a: 42)
    result = UniffiBindgenTests.roundtrip_ext_record(rec)
    assert_equal 42, result.a
  end

  def test_ext_enum
    result = UniffiBindgenTests.roundtrip_ext_enum(Ext::ExternalEnum::TWO)
    assert_equal Ext::ExternalEnum::TWO, result
  end

  def test_ext_interface
    obj = Ext::ExternalInterface.new(123)
    result = UniffiBindgenTests.roundtrip_ext_interface(obj)
    assert_equal 123, result.get_value
  end

  def test_ext_custom_type
    result = UniffiBindgenTests.roundtrip_ext_custom_type(789)
    assert_equal 789, result
  end

  # Identity imported u64 newtypes skip consumer coerce; defining-crate
  # `uniffi_lower_*` must still run `uniffi_in_range` (including `to_int`).
  def test_ext_custom_type_rejects_negative_like_local
    local = assert_raises(RangeError) { UniffiBindgenTests.roundtrip_custom_type1(-1) }
    imported = assert_raises(RangeError) { UniffiBindgenTests.roundtrip_ext_custom_type(-1) }

    assert_equal local.message, imported.message
    assert_equal "u64 requires 0 <= value < #{2**64}", imported.message
  end

  def test_ext_custom_type_rejects_non_integer_like_local
    local = assert_raises(TypeError) { UniffiBindgenTests.roundtrip_custom_type1('nope') }
    imported = assert_raises(TypeError) { UniffiBindgenTests.roundtrip_ext_custom_type('nope') }

    assert_equal local.message, imported.message
    assert_equal 'no implicit conversion of nope into Integer', imported.message
  end

  # Ruby 4 Float#to_int truncates (1.9 -> 1). Both paths must agree; FFI
  # `:uint64` without `uniffi_in_range` would not go through `to_int`.
  def test_ext_custom_type_coerces_float_like_local
    assert_equal 1, UniffiBindgenTests.roundtrip_custom_type1(1.9)
    assert_equal 1, UniffiBindgenTests.roundtrip_ext_custom_type(1.9)
  end

  def test_ext_custom_type_preserves_to_int_coercion
    int_like = Object.new
    def int_like.to_int
      7
    end

    assert_equal 7, UniffiBindgenTests.roundtrip_custom_type1(int_like)
    assert_equal 7, UniffiBindgenTests.roundtrip_ext_custom_type(int_like)
  end

  def test_ext_url_is_uri
    url = URI.parse('http://example.com/')
    result = UniffiBindgenTests.roundtrip_ext_url(url)

    assert_kind_of URI, result
    assert_equal url, result
  end

  def test_local_url_wrapping_imported_url
    url = URI.parse('http://example.com/local')
    result = UniffiBindgenTests.roundtrip_local_url(url)

    assert_kind_of URI, result
    assert_equal url, result
  end

  def test_ext_nested_rec
    rec = Ext::ExternalNestedRec.new(
      en: Ext::ExternalEnum::ONE,
      rec: Ext::ExternalRec.new(a: 7)
    )
    result = UniffiBindgenTests.roundtrip_ext_nested_rec(rec)

    assert_equal Ext::ExternalEnum::ONE, result.en
    assert_equal 7, result.rec.a
  end

  def test_nested_ext_rec_identity_custom
    inner = Ext::ExternalRec.new(a: 9)
    result = UniffiBindgenTests.roundtrip_nested_ext_rec(inner)

    assert_instance_of Ext::ExternalRec, result
    assert_equal 9, result.a
  end

  def test_nested_ext_interface_identity_custom
    obj = Ext::ExternalInterface.new(3)
    result = UniffiBindgenTests.roundtrip_nested_ext_interface(obj)

    assert_instance_of Ext::ExternalInterface, result
    assert_equal 3, result.get_value
  end

  def test_optional_and_sequence
    assert_nil UniffiBindgenTests.roundtrip_maybe_ext_enum(nil)
    assert_equal Ext::ExternalEnum::TWO,
                 UniffiBindgenTests.roundtrip_maybe_ext_enum(Ext::ExternalEnum::TWO)
    assert_equal [Ext::ExternalEnum::ONE, Ext::ExternalEnum::THREE],
                 UniffiBindgenTests.roundtrip_ext_enums(
                   [Ext::ExternalEnum::ONE, Ext::ExternalEnum::THREE]
                 )
  end

  def test_async_ext_enum
    result = UniffiBindgenTests.async_roundtrip_ext_enum(Ext::ExternalEnum::TWO)
    assert_equal Ext::ExternalEnum::TWO, result
  end

  def test_throw_ext_error
    assert_raise Ext::ExternalError::Boom do
      UniffiBindgenTests.throw_ext_error
    end
  end

  def test_corrupt_external_enum_raises_defining_crate_internal_error
    buf = UniffiBindgenTests::RustBuffer.alloc(4)
    buf.len = 4
    buf.data.put_bytes(0, [99].pack('l>'))
    err = assert_raise(Ext::InternalError) do
      buf.consume_into_TypeExternalEnum
    end

    assert_match(/Unexpected variant tag/, err.message)
    assert_not_same UniffiBindgenTests::InternalError, Ext::InternalError
  end

  def test_mid_rec_roundtrip
    rec = Mid::MidRec.new(
      inner: Ext::ExternalRec.new(a: 11),
      maybe_enum: Ext::ExternalEnum::ONE
    )
    result = UniffiBindgenTests.roundtrip_mid_rec(rec)

    assert_equal 11, result.inner.a
    assert_equal Ext::ExternalEnum::ONE, result.maybe_enum
  end

  # MidRec's nested External* fields are not in the consumer CI. Mid's generated
  # file must still require the source crate and name its mixin.
  def test_mid_generated_requires_source
    path = $LOADED_FEATURES.find { |f| File.basename(f) == 'uniffi_bindgen_tests_mid_types.rb' }
    assert_not_nil path, 'uniffi_bindgen_tests_mid_types.rb should be loaded'
    src = File.read(path)

    assert_match(/require ['"]uniffi_bindgen_tests_external_types_source['"]/, src)
    assert_match(/UniffiBindgenTestsExternalTypesSource::RustBuffer/, src)
  end
end
