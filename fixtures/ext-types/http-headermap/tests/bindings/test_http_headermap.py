# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from ext_types_http_headermap import *

class TestCallback(HeaderMapCallback):
    def run(self, header_map):
        self.saw_map = header_map
        copy = header_map.copy()
        copy.append(HttpHeader(key="from-callback", val="hello"))
        return copy

class TestHttpHeaderMap(unittest.TestCase):
    def test_get_headermap(self):
        self.assertEqual(get_headermap("Second value"), [
            HttpHeader(key="test-header", val="First value"),
            HttpHeader(key="test-header", val="Second value")
        ])

    def test_guid_callback(self):
        # Test that we can passing a guid from run_callback() to TestCallback.run() then back out

        test_callback = TestCallback()
        m = run_callback(test_callback)
        self.assertEqual(m, [
            HttpHeader(key="foo", val="bar"),
            HttpHeader(key="from-callback", val="hello")
        ])
        self.assertEqual(test_callback.saw_map, [
            HttpHeader(key="foo", val="bar"),
        ])

if __name__=='__main__':
    unittest.main()
