# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import primitive_types
import unittest

class FnCallsTest(unittest.TestCase):
    def test_roundtrip(self):
        self.assertEqual(primitive_types.roundtrip(0), 0)
        self.assertEqual(primitive_types.roundtrip(42), 42)

    def test_roundtrip_bool(self):
        self.assertEqual(primitive_types.roundtrip_bool(True), True)
        self.assertEqual(primitive_types.roundtrip_bool(False), False)

    def test_roundtrip_string(self):
        self.assertEqual(primitive_types.roundtrip_string("Hello"), "Hello")

    def test_sum(self):
        self.assertEqual(primitive_types.sum(1, -1, 2, -2, 3, -3, 4, -4, 0.5, 1.5, True), -2.0)

unittest.main()
