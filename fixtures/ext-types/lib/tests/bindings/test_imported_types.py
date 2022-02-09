# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from imported_types_lib import *

class TestIt(unittest.TestCase):
    def test_it(self):
        ct = get_combined_type(None)
        self.assertEqual(ct.uot.sval, "hello")
        self.assertEqual(ct.guid, "a-guid")
        self.assertEqual(ct.url.scheme, 'http')
        self.assertEqual(ct.url.netloc, 'example.com')
        self.assertEqual(ct.url.path, '/')

        ct2 = get_combined_type(ct)
        self.assertEqual(ct, ct2)


if __name__=='__main__':
    unittest.main()
