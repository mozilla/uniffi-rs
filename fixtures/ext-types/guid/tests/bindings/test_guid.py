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

    def test_get_guid_errors(self):
        # This is testing `get_guid` which never returns a result, so everything
        # is InternalError representing a panic.
        # The fixture hard-codes some Guid strings to return specific errors.
        with self.assertRaisesRegex(InternalError, "Failed to convert arg 'val': The Guid is too short"):
            get_guid("")

        with self.assertRaisesRegex(InternalError, "Failed to convert arg 'val': Something unexpected went wrong"):
            get_guid("unexpected")

        with self.assertRaisesRegex(InternalError, "guid value caused a panic!"):
            get_guid("panic")

    def test_try_get_guid_errors(self):
        # This is testing `try_get_guid()` which says it returns a result, so we
        # will get a mix of "expected" errors and panics.
        with self.assertRaises(GuidError.TooShort):
            try_get_guid("")

        with self.assertRaisesRegex(InternalError, "Failed to convert arg 'val': Something unexpected went wrong"):
            try_get_guid("unexpected")

        with self.assertRaisesRegex(InternalError, "guid value caused a panic!"):
            try_get_guid("panic")

    # def test_round_trip(self):
    #     ct = get_combined_type(None)
    #     self.assertEqual(ct.cot.sval, "hello")
    #     self.assertEqual(ct.ctt.ival, 1)

if __name__=='__main__':
    unittest.main()
