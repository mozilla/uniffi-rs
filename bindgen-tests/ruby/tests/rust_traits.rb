# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_bindgen_tests'

class TestRustTraits < Test::Unit::TestCase
  include UniffiBindgenTests

  def test_debug
    trait_test = RustTraitTest.new(a: 1, b: 2)
    assert_equal 'debug-test-string', trait_test.inspect
  end

  def test_display
    trait_test = RustTraitTest.new(a: 1, b: 2)
    assert_equal 'display-test-string', trait_test.to_s
  end

  def test_eq
    # The Rust code only uses `a` for the equality
    assert_equal RustTraitTest.new(a: 1, b: 2), RustTraitTest.new(a: 1, b: 3)
    assert_not_equal RustTraitTest.new(a: 2, b: 2), RustTraitTest.new(a: 1, b: 2)
  end

  def test_ord
    # The Rust code only uses `a` for the ordering
    assert RustTraitTest.new(a: 1, b: 2) < RustTraitTest.new(a: 2, b: 3)
    assert RustTraitTest.new(a: 2, b: 3) > RustTraitTest.new(a: 1, b: 2)
    assert RustTraitTest.new(a: 1, b: 2) <= RustTraitTest.new(a: 1, b: 2)
    assert RustTraitTest.new(a: 1, b: 2) >= RustTraitTest.new(a: 1, b: 2)
  end

  def test_hash
    # The Rust code only uses `a` for the hash
    assert_equal RustTraitTest.new(a: 1, b: 2).hash, RustTraitTest.new(a: 1, b: 2).hash
    assert_not_equal RustTraitTest.new(a: 1, b: 2).hash, RustTraitTest.new(a: 2, b: 2).hash
  end

  def test_hash_fn
    # rust_trait_test_hash should match Ruby's hash based on `a` only
    assert_equal(
      UniffiBindgenTests.rust_trait_test_hash(RustTraitTest.new(a: 1, b: 2)), 
      UniffiBindgenTests.rust_trait_test_hash(RustTraitTest.new(a: 1, b: 3))
    )

    assert_not_equal(
      UniffiBindgenTests.rust_trait_test_hash(RustTraitTest.new(a: 1, b: 2)), 
      UniffiBindgenTests.rust_trait_test_hash(RustTraitTest.new(a: 2, b: 2))
    )
  end
end
