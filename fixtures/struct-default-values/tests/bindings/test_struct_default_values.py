# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from struct_default_values import *

class TestStructDefaultValues(unittest.TestCase):
    def test_bookmark_only_nondefault_set(self):
        url = "https://mozilla.github.io/uniffi-rs"
        bookmark = Bookmark(position=2, url=url)

        assert not bookmark.guid
        assert bookmark.position == 2
        assert bookmark.url == url

    def test_bookmark_others_set(self):
        url = "https://mozilla.github.io/uniffi-rs"
        bookmark = Bookmark(position=3, url=url, guid="c0ffee")

        assert bookmark.guid == "c0ffee"
        assert bookmark.position == 3
        assert bookmark.url == url

if __name__=='__main__':
    unittest.main()
