# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from test_package.uniffi_bindgen_tests import *
import unittest

class TestUdl(unittest.TestCase):
    def test_udl(self):
        self.assertEqual(udl_sum_fields(UdlType(a=0.5, b=1.5)), 2.0)
        self.assertEqual(udl_make_type(0.5, 1.5), UdlType(a=0.5, b=1.5))
        self.assertEqual(udl_remote_sum_fields(UdlRemoteType(a=0.5, b=1.5)), 2.0)

if __name__ == '__main__':
    unittest.main()
