# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
import uniffi_one
from imported_types_lib import *
from ext_types_guid import Guid

class TestIt(unittest.TestCase):
    def test_it(self):
        self.assertEqual(get_uniffi_one_type(uniffi_one.UniffiOneType("test")).sval, "test - test")

        ct = get_combined_type(None)
        self.assertEqual(ct.uot.sval, "hello")
        self.assertEqual(ct.guid, Guid("a-guid"))
        self.assertEqual(ct.json, '{"hello":"there"}')

        ct2 = get_combined_type(ct)
        self.assertEqual(ct, ct2)


if __name__=='__main__':
    unittest.main()
