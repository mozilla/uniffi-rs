# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestCollections < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_vecs
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i8([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u16([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i16([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u32([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i32([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_u64([1, 2, 3])
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec_i64([1, 2, 3])
    assert_equal ['test-string'], UniffiBindgenTests.roundtrip_vec_string(['test-string'])
    assert_equal [true, false], UniffiBindgenTests.roundtrip_vec_bool([true, false])

    assert_equal(
      [CollectionsRec.new(a: 42)],
      UniffiBindgenTests.roundtrip_vec_rec([CollectionsRec.new(a: 42)])
    )
  end

  def test_hash_map
    map = { 'a' => 1, 'b' => 2 }

    assert_equal map, UniffiBindgenTests.roundtrip_hash_map(map)
    assert_equal({}, UniffiBindgenTests.roundtrip_hash_map({}))

    assert_equal(
      { 1 => 2, 2 => 4 },
      UniffiBindgenTests.roundtrip_hash_map_u32_key({ 1 => 2, 2 => 4 })
    )
  end

  def test_hash_set
    set = Set.new(%w[a b c])

    assert_equal set, UniffiBindgenTests.roundtrip_hash_set(set)
    assert_equal(Set.new([]), UniffiBindgenTests.roundtrip_hash_set(Set.new([])))

    assert_equal [Set.new(%w[a b c])], UniffiBindgenTests.roundtrip_vec_hash_set([set])
    assert_equal nil, UniffiBindgenTests.roundtrip_vec_hash_set(nil)
  end

  def test_rec_with_collections
    assert_equal(
      record_with_collection,
      UniffiBindgenTests.roundtrip_rec_with_collections(record_with_collection)
    )
  end

  def test_complex_collection_type
    assert_equal(
      complex_collection_type,
      UniffiBindgenTests.roundtrip_complex_collection_type(complex_collection_type)
    )

    assert_nil UniffiBindgenTests.roundtrip_complex_collection_type(nil)
  end

  def record_with_collection
    RecWithCollections.new(
      a: EnumWithCollections::A.new(nil),
      b: nil,
      c: [true, false],
      d: { 'a' => 1, 'b' => 2 }
    )
  end

  def complex_collection_type
    [{
      'a' => CollectionsComplexRec.new(a: 10, b: 'test', c: CollectionsEnum::A.new(100)),
      'b' => CollectionsComplexRec.new(a: 20, b: 'test2', c: CollectionsEnum::B.new(a: 1.0, b: true))
    }]
  end
end
