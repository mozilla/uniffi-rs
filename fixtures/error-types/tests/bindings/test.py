# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from error_types import *

class TestErrorTypes(unittest.TestCase):
    def test_normal_catch(self):
        try:
            anyhow_bail("oh no")
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "oh no")

    def test_interface_errors(self):
        with self.assertRaises(ErrorInterface) as cm:
            anyhow_with_context("oh no")
        self.assertEqual(cm.exception.chain(), ["because uniffi told me so", "oh no"])
        self.assertEqual(cm.exception.link(0), "because uniffi told me so")
        self.assertEqual(repr(cm.exception), "ErrorInterface { e: because uniffi told me so\n\nCaused by:\n    oh no }")
        self.assertEqual(str(cm.exception), "because uniffi told me so")

    # Check we can still call a function which returns an error (as opposed to one which throws it)
    def test_error_return(self):
        e = get_error("the error")
        self.assertEqual(e.chain(), ["the error"])
        self.assertEqual(repr(e), "ErrorInterface { e: the error }")
        self.assertEqual(str(e), "the error")

    # RichError is not an anyhow error.
    def test_rich_error(self):
        try:
            throw_rich("oh no")
            self.fail("must fail")
        except RichError as e:
           self.assertEqual(repr(e), """RichError { e: "oh no" }""")
           self.assertEqual(str(e), "") # XXX - this sucks?

    def test_rich_error_return(self):
        e = get_rich_error("the error")
        self.assertEqual(repr(e), """RichError { e: "the error" }""")
        self.assertEqual(str(e), "") # XXX - this sucks?

    # TestInterface also throws.
    def test_interface_errors(self):
        with self.assertRaises(ErrorInterface) as cm:
            TestInterface.fallible_new()
        self.assertEqual(str(cm.exception), "fallible_new")

        interface = TestInterface()
        with self.assertRaises(ErrorInterface) as cm:
            interface.anyhow_bail("oops")
        self.assertEqual(str(cm.exception), "TestInterface - oops")

    # TestInterface also throws.
    def test_interface_errors(self):
        with self.assertRaises(ProcErrorInterface) as cm:
            throw_proc_error("eek")
        self.assertEqual(cm.exception.message(), "eek")
# No UniffiTrait support yet, so no __str__ etc.
#        self.assertEqual(str(cm.exception), "eek")

if __name__=='__main__':
    unittest.main()
