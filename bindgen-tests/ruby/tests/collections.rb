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
    assert_equal ["test-string"], UniffiBindgenTests.roundtrip_vec_string(["test-string"])
    assert_equal [true, false], UniffiBindgenTests.roundtrip_vec_bool([true, false])

    rec = CollectionsRec.new(a: 42)
    assert_equal [rec], UniffiBindgenTests.roundtrip_vec_rec([rec])
  end

  def test_hash_map
    map = { 'a' => 1, 'b' => 2 }

    assert_equal map, UniffiBindgenTests.roundtrip_hash_map(map)
    assert_equal({}, UniffiBindgenTests.roundtrip_hash_map({}))
  end

  def test_rec_with_collections
    rec = RecWithCollections.new(
      a: EnumWithCollections::A.new(nil),
      b: nil,
      c: [true, false],
      d: { 'a' => 1, 'b' => 2 },
    )

    assert_equal rec, UniffiBindgenTests.roundtrip_rec_with_collections(rec)
  end

  def test_complex_collection_type
    inner = [{
      'a' => CollectionsComplexRec.new(a: 10, b: "test", c: CollectionsEnum::A.new(100)),
      'b' => CollectionsComplexRec.new(a: 20, b: "test2", c: CollectionsEnum::B.new(a: 1.0, b: true)),
    }]
    assert_equal inner, UniffiBindgenTests.roundtrip_complex_collection_type(inner)
    assert_nil UniffiBindgenTests.roundtrip_complex_collection_type(nil)
  end
end
