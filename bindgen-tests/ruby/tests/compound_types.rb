# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestCompoundTypes < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_option_name
    assert_equal 42, UniffiBindgenTests.roundtrip_option(42)
  end

  def test_option_none
    assert_nil UniffiBindgenTests.roundtrip_option(nil)
  end

  def test_vec
    assert_equal [1, 2, 3], UniffiBindgenTests.roundtrip_vec([1, 2, 3])
    assert_equal [], UniffiBindgenTests.roundtrip_vec([])
  end

  def test_hash_map
    map = { 'a' => 1, 'b' => 2 }

    assert_equal map, UniffiBindgenTests.roundtrip_hash_map(map)
    assert_equal({}, UniffiBindgenTests.roundtrip_hash_map({}))
  end

  def test_complex_compound_some
    inner = [{ 'x' => 10, 'y' => 20 }]

    assert_equal inner, UniffiBindgenTests.roundtrip_complex_compound(inner)
  end

  def test_complex_compound_none
    assert_nil UniffiBindgenTests.roundtrip_complex_compound(nil)
  end
end
