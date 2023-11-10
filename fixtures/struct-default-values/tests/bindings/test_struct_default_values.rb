# frozen_string_literal: true

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

require 'test/unit'
require 'struct_default_values'

include StructDefaultValues

class TestStructDefaultValues < Test::Unit::TestCase

  def test_bookmark_only_nondefault_set
    url = "https://mozilla.github.io/uniffi-rs"
    bookmark = Bookmark.new(position: 2, url: url)

    assert_nil bookmark.guid
    assert_equal bookmark.position, 2
    assert_equal bookmark.url, url
  end

  def test_bookmark_others_set()
    url = "https://mozilla.github.io/uniffi-rs"
    bookmark = Bookmark.new(position: 3, url: url, guid: "c0ffee")

    assert_equal bookmark.guid, "c0ffee"
    assert_equal bookmark.position, 3
    assert_equal bookmark.url, url
  end

  def test_order_doesnt_matter()
    url = "https://mozilla.github.io/uniffi-rs"
    bookmark = Bookmark.new(url: url, guid: "c0ffee", position: 3)

    assert_equal bookmark.guid, "c0ffee"
    assert_equal bookmark.position, 3
    assert_equal bookmark.url, url
  end

  def test_unnamed_unsupported()
    assert_raise ArgumentError do
      bookmark = Bookmark.new(3, "https://mozilla.github.io/uniffi-rs")
    end
  end

end
