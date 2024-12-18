# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import fn_calls
import unittest

class FnCallsTest(unittest.TestCase):
    def test_fn_call(self):
        fn_calls.test_func()

unittest.main()
