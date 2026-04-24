# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uri'
require 'uniffi_fixture_rename'

class TestRename < Test::Unit::TestCase
  def test_rename_record
    record = UniffiFixtureRename::RenamedRecord.new renamed_field: 42
    assert_equal 42, record.renamed_field
  end

  def test_renamed_enum
    variant_a = UniffiFixtureRename::RenamedEnum::RENAMED_VARIANT.new
    assert variant_a.renamed_variant?
    assert !variant_a.record?

    record = UniffiFixtureRename::RenamedRecord.new renamed_field: 42
    variant_record = UniffiFixtureRename::RenamedEnum::RECORD.new record
    assert !variant_record.renamed_variant?
    assert variant_record.record?
    assert_equal 42, variant_record[0].renamed_field
  end

  def test_renamed_enum_with_fields
    variant = UniffiFixtureRename::RenamedEnumWithFields::RENAMED_VARIANT_WITH_FIELDS.new(
      renamed_variant_field: 42
    )

    assert variant.renamed_variant_with_fields?
    assert_equal 42, variant.renamed_variant_field
  end

  def test_renamed_function
    record = UniffiFixtureRename::RenamedRecord.new renamed_field: 42
    result = UniffiFixtureRename.renamed_function record

    assert_instance_of UniffiFixtureRename::RenamedEnum::RECORD, result
    assert_equal 42, result[0].renamed_field
  end

  def test_renamed_object
    renamed_object = UniffiFixtureRename::RenamedObject.renamed_constructor 42

    assert_instance_of UniffiFixtureRename::RenamedObject, renamed_object
    assert_equal 42, renamed_object.renamed_method
  end

  def test_renamed_error
    assert UniffiFixtureRename::RenamedError::RenamedErrorVariant < StandardError
  end

  def test_renamed_trait_method
    impl = UniffiFixtureRename.create_trait_impl 5

    assert_equal 50, impl.renamed_trait_method(10)
  end

  def test_roundtrip_url
    url = URI.parse 'https://example.com/test'
    result = UniffiFixtureRename.roundtrip_url url

    assert_instance_of URI::HTTPS, result
    assert_equal url, result
  end
end

class TestBindingRenames < Test::Unit::TestCase
  def test_rb_record
    record = UniffiFixtureRename::RbRecord.new rb_item: 42

    assert_equal 42, record.rb_item
  end

  def test_rb_enum
    record = UniffiFixtureRename::RbRecord.new rb_item: 42

    variant_a = UniffiFixtureRename::RbEnum::RB_VARIANT_A.new
    assert variant_a.rb_variant_a?

    variant_record = UniffiFixtureRename::RbEnum::RB_RECORD.new record
    assert variant_record.rb_record?
    assert_equal 42, variant_record[0].rb_item
  end

  def test_rb_enum_with_fields
    variant_a = UniffiFixtureRename::RbEnumWithFields::RB_VARIANT_A.new rb_int: 42
    assert variant_a.rb_variant_a?
    assert_equal 42, variant_a.rb_int

    inner = UniffiFixtureRename::RbRecord.new rb_item: 7
    variant_rec = UniffiFixtureRename::RbEnumWithFields::RB_RECORD.new rb_record: inner, rb_int: 3
    
    assert variant_rec.rb_record?
    assert_equal inner, variant_rec.rb_record
    assert_equal 3, variant_rec.rb_int
  end

  def test_rb_function_with_record
    record = UniffiFixtureRename::RbRecord.new rb_item: 99
    result = UniffiFixtureRename.rb_function record

    assert_instance_of UniffiFixtureRename::RbEnum::RB_RECORD, result
    assert_equal 99, result[0].rb_item
  end

  def test_rb_function_with_nil_raises
    assert_raise(UniffiFixtureRename::RbError::RbSimple) { UniffiFixtureRename.rb_function nil }
  end

  def test_rb_object
    obj = UniffiFixtureRename::RbObject.new 7

    assert_equal 17, obj.rb_method(10)
  end

  def test_rb_trait
    impl = UniffiFixtureRename.create_binding_trait_impl 3

    assert_equal 30, impl.rb_trait_method(10)
  end

  def test_exclusions
    assert !UniffiFixtureRename.respond_to?(:function_to_exclude)
    assert !defined?(UniffiFixtureRename::RecordToExclude)

    obj = UniffiFixtureRename::RenamedObject.renamed_constructor 42
    assert !obj.respond_to?(:method_to_exclude)
  end
end
