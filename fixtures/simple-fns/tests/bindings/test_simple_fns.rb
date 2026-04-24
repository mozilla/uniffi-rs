# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

require 'test/unit'
require 'uniffi_simple_fns'

class TestSimpleFns < Test::Unit::TestCase
  def test_simple_fns
    assert_equal 'String created by Rust', UniffiSimpleFns.get_string
    assert_equal 1289, UniffiSimpleFns.get_int
    assert_equal 'String created by Ruby', UniffiSimpleFns.string_identity('String created by Ruby')
    assert_equal 255, UniffiSimpleFns.byte_to_u32(255)

    a_set = UniffiSimpleFns.new_set
    UniffiSimpleFns.add_to_set a_set, 'foo'
    UniffiSimpleFns.add_to_set a_set, 'bar'

    assert UniffiSimpleFns.set_contains(a_set, 'foo')
    assert UniffiSimpleFns.set_contains(a_set, 'bar')
    assert !UniffiSimpleFns.set_contains(a_set, 'baz')

    assert_equal({'a' => 'b'}, UniffiSimpleFns.hash_map_identity({'a' => 'b'}))
  end
end
