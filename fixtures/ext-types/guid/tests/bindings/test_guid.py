# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from ext_types_guid import *

class TestGuid(unittest.TestCase):
    def test_get_guid(self):
        self.assertEqual(get_guid(None), "NewGuid")
        self.assertEqual(get_guid("SomeGuid"), "SomeGuid")

    def test_guid_helper(self):
        helper = get_guid_helper(None)
        self.assertEqual(helper.guid, "first-guid")
        self.assertEqual(helper.guids, ["second-guid", "third-guid"])
        self.assertEqual(helper.maybe_guid, None)

    def test_guid_errors(self):
        self.assertRaises(GuidError.TooShort, get_guid, "")

    # def test_round_trip(self):
    #     ct = get_combined_type(None)
    #     self.assertEqual(ct.cot.sval, "hello")
    #     self.assertEqual(ct.ctt.ival, 1)

if __name__=='__main__':
    unittest.main()
