# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestRenames < Test::Unit::TestCase
  include UniffiBindgenTests

  # Rust-side renames via #[uniffi(name = ...)] ---

  def test_rename_record
    rec = RenamedRecord.new item: 42
    assert_equal 42, rec.item
  end

  def test_rename_enum
    rec = RenamedRecord.new item: 42
    result = UniffiBindgenTests.renamed_function rec

    assert_kind_of RenamedEnum::RECORD, result
    assert_equal rec.item, result[0].item
  end

  def test_renamed_enum_variant
    variant = RenamedEnum::RENAMED_VARIANT.new

    assert_kind_of RenamedEnum::RENAMED_VARIANT, variant
  end

  def test_rename_object
    obj = RenamedObject.renamed_constructor 123

    assert_equal 123, obj.renamed_method
  end

  def test_rename_trait
    impl = UniffiBindgenTests.create_trait_impl 5

    assert_equal 50, impl.renamed_trait_method(10)
  end

  # --- Binding-specific renames via uniffi.toml [bindings.ruby.rename] ---

  def test_rb_record
    rec = RbRecord.new rb_item: 100

    assert_equal 100, rec.rb_item
  end

  def test_rb_enum
    assert_kind_of RbEnum::RB_VARIANT_A, RbEnum::RB_VARIANT_A.new

    rec = RbRecord.new rb_item: 7
    result = RbEnum::RB_RECORD.new rec

    assert_equal 7, result[0].rb_item
  end

  def test_rb_enum_with_fields
    va = RbEnumWithFields::RB_VARIANT_A.new rb_int: 5
    assert_equal 5, va.rb_int

    rec = RbRecord.new rb_item: 3
    vr = RbEnumWithFields::RB_RECORD.new rb_record: rec, rb_int: 9

    assert_equal 3, vr.rb_record.rb_item
    assert_equal 9, vr.rb_int
  end

  def test_rb_function
    rec = RbRecord.new rb_item: 100
    result = UniffiBindgenTests.rb_function rec

    assert_kind_of RbEnum::RB_RECORD, result
    assert_equal 100, result[0].rb_item
  end

  def test_rb_function_raises
    assert_raises(RbError::RbSimple) { UniffiBindgenTests.rb_function nil }
  end

  def test_rb_object
    obj = RbObject.new 200

    assert_equal 250, obj.rb_method(50)
  end

  def test_rb_trait
    impl = UniffiBindgenTests.create_binding_trait_to_rename_impl 3

    assert_equal 21, impl.rb_trait_method(7)
  end
end
