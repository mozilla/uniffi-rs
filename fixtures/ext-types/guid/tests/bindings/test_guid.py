# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from ext_types_guid import *

class TestGuid(unittest.TestCase):
    def test_get_guid(self):
        self.assertEqual(get_guid(None), Guid("NewGuid"))
        self.assertEqual(get_guid(Guid("SomeGuid")), Guid("SomeGuid"))

    def test_guid_helper(self):
        helper = get_guid_helper(None)
        self.assertEqual(helper.guid, Guid("first-guid"))
        self.assertEqual(helper.guids, [Guid("second-guid"), Guid("third-guid")])
        self.assertEqual(helper.maybe_guid, None)

    def test_rust_guid(self):
        self.assertEqual(get_rust_guid(), "RustGuid")

    def test_python_guid(self):
        self.assertEqual(get_python_guid(), PythonGuid("PythonGuid"))

if __name__=='__main__':
    unittest.main()
