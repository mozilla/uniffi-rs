# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from external_types_lib import *

class TestExternalTypes(unittest.TestCase):
    def test_round_trip(self):
        ct = get_combined_type(CombinedType(
            CrateOneType("test"),
            CrateTwoType(42),
        ))
        self.assertEqual(ct.cot.sval, "test")
        self.assertEqual(ct.ctt.ival, 42)

    def test_none_value(self):
        ct = get_combined_type(None)
        self.assertEqual(ct.cot.sval, "hello")
        self.assertEqual(ct.ctt.ival, 1)

if __name__=='__main__':
    unittest.main()
