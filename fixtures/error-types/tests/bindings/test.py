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

    def test_enum_error(self):
        with self.assertRaises(Error) as cm:
            oops_enum(0)
        # This doesn't seem ideal?
        self.assertEqual(str(cm.exception), "")
        self.assertEqual(repr(cm.exception), "Error.Oops()")

    def test_enum_error_value(self):
        with self.assertRaises(Error) as cm:
            oops_enum(1)
        self.assertEqual(str(cm.exception), "value='value'")
        self.assertEqual(repr(cm.exception), "Error.Value(value='value')")

    def test_enum_error_int_value(self):
        with self.assertRaises(Error) as cm:
            oops_enum(2)
        self.assertEqual(str(cm.exception), "value=2")
        self.assertEqual(repr(cm.exception), "Error.IntValue(value=2)")

    def test_enum_flat_inner(self):
        with self.assertRaises(Error.FlatInnerError) as cm:
            oops_enum(3)
        # XXX - can't compare Python errors.
        self.assertEqual(str(cm.exception.error), "inner")

        with self.assertRaises(Error.FlatInnerError) as cm:
            oops_enum(4)
        # XXX - can't compare Python errors.
        self.assertEqual(str(cm.exception.error), "NonUniffiTypeValue: value")

    def test_enum_inner(self):
        with self.assertRaises(Error.InnerError) as cm:
            oops_enum(5)
        # XXX - can't compare Python errors.
        self.assertEqual(cm.exception.error[0], "inner")

    def test_tuple_error(self):
        r = get_tuple()
        self.assertEqual(repr(r), "TupleError.Oops('oops')")
        # self.assertEqual(get_tuple(r), r)
        with self.assertRaises(TupleError) as cm:
            oops_tuple(0)
        self.assertEqual(str(cm.exception), "'oops'")
        self.assertEqual(repr(cm.exception), "TupleError.Oops('oops')")

        with self.assertRaises(TupleError) as cm:
            oops_tuple(1)
        self.assertEqual(str(cm.exception), "1")
        self.assertEqual(repr(cm.exception), "TupleError.Value(1)")

if __name__=='__main__':
    unittest.main()
