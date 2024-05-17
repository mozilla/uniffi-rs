# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import asyncio
import unittest
from error_types import *

class TestErrorTypes(unittest.TestCase):
    def test_normal_catch(self):
        try:
            oops()
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "because uniffi told me so\n\nCaused by:\n    oops")

    def test_normal_catch_with_implit_arc_wrapping(self):
        try:
            oops_nowrap()
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "because uniffi told me so\n\nCaused by:\n    oops")

    def test_error_interface(self):
        with self.assertRaises(ErrorInterface) as cm:
            oops()
        self.assertEqual(cm.exception.chain(), ["because uniffi told me so", "oops"])
        self.assertEqual(cm.exception.link(0), "because uniffi told me so")
        self.assertEqual(repr(cm.exception), "ErrorInterface { e: because uniffi told me so\n\nCaused by:\n    oops }")
        self.assertEqual(str(cm.exception), "because uniffi told me so\n\nCaused by:\n    oops")

    def test_async_error_interface(self):
        try:
            asyncio.run(aoops())
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "async-oops")

    def test_error_trait(self):
        with self.assertRaises(ErrorTrait) as cm:
            toops()
        self.assertEqual(cm.exception.msg(), "trait-oops")

    # Check we can still call a function which returns an error (as opposed to one which throws it)
    def test_error_return(self):
        e = get_error("the error")
        self.assertEqual(e.chain(), ["the error"])
        self.assertEqual(repr(e), "ErrorInterface { e: the error }")
        self.assertEqual(str(e), "the error")

    def test_rich_error(self):
        try:
            throw_rich("oh no")
            self.fail("must fail")
        except RichError as e:
           self.assertEqual(repr(e), 'RichError { e: "oh no" }')
           self.assertEqual(str(e), 'RichError: "oh no"')

    # TestInterface also throws.
    def test_interface_errors(self):
        with self.assertRaises(ErrorInterface) as cm:
            TestInterface.fallible_new()
        self.assertEqual(str(cm.exception), "fallible_new")

        interface = TestInterface()
        with self.assertRaises(ErrorInterface) as cm:
            interface.oops()
        self.assertEqual(str(cm.exception), "because the interface told me so\n\nCaused by:\n    oops")

        try:
            asyncio.run(interface.aoops())
            self.fail("must fail")
        except ErrorInterface as e:
           self.assertEqual(str(e), "async-oops")

    # TestInterface also throws.
    def test_procmacro_interface_errors(self):
        with self.assertRaises(ProcErrorInterface) as cm:
            throw_proc_error("eek")
        self.assertEqual(cm.exception.message(), "eek")
        self.assertEqual(str(cm.exception), "ProcErrorInterface(eek)")

    def test_enum_errors(self):
        with self.assertRaises(Error.Oops) as cm:
            oops_enum()
        # should be able to see "Oops", right!?
        #?? self.assertEqual(cm.exception.args[0], "Oops")

    def test_enum_flat_inner(self):
        with self.assertRaises(Error.Oops) as cm:
            oops_flat_inner()
        # check value?

if __name__=='__main__':
    unittest.main()
