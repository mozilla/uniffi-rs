/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.struct_default_values.*

// TODO: use an actual test runner.

val url = "https://mozilla.github.io/uniffi-rs";

var bookmark = Bookmark(position=2, url=url)
assert(bookmark.guid == null)
assert(bookmark.position == 2)
assert(bookmark.url == url)

bookmark = Bookmark(position=3, url=url, guid="c0ffee")
assert(bookmark.guid == "c0ffee")
assert(bookmark.position == 3)
assert(bookmark.url == url)

// Order doesn't matter here.
bookmark = Bookmark(url=url, guid="c0ffee", position=4)
assert(bookmark.guid == "c0ffee")
assert(bookmark.position == 4)
assert(bookmark.url == url)
assert(bookmark.lastModified == null)
assert(bookmark.title == null)

// Order matters here when unnamed.
bookmark = Bookmark("c0ffee", 5, 17, url)
assert(bookmark.guid == "c0ffee")
assert(bookmark.position == 5)
assert(bookmark.url == url)
assert(bookmark.lastModified == 17)
assert(bookmark.title == null)
