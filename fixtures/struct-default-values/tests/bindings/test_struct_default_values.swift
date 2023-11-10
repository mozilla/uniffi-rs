/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import struct_default_values

// TODO: use an actual test runner.

let url = "https://mozilla.github.io/uniffi-rs";

var bookmark = Bookmark(position: 2, url: url)
assert(bookmark.guid == nil)
assert(bookmark.position == 2)
assert(bookmark.url == url)

// In Swift order of named arguments still matters.
bookmark = Bookmark(guid: "c0ffee", position: 3, url: url)
assert(bookmark.guid == "c0ffee")
assert(bookmark.position == 3)
assert(bookmark.url == url)

// No unnamed parameters allowed here,
// so we cannot test that
