# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
from uniffi_fixture_rename import *

class TestRename(unittest.TestCase):
    def test_rename(self):
        # Test renamed record
        record = RenamedRecord(renamed_field=42)
        self.assertEqual(record.renamed_field, 42)

        # Test renamed enum
        enum1 = RenamedEnum.RENAMED_VARIANT()
        enum2 = RenamedEnum.RECORD(record)
        self.assertEqual(enum2[0].renamed_field, 42)

        # Test renamed function
        result = renamed_function(record)
        self.assertEqual(result[0].renamed_field, 42)

        # Test renamed object with renamed constructor and method
        obj = RenamedObject.renamed_constructor(123)
        self.assertIsInstance(obj, RenamedObject)
        self.assertEqual(obj.renamed_method(), 123)

        # Test renamed error type exists
        self.assertTrue(hasattr(RenamedError, 'RenamedErrorVariant'))

        # Test trait method renaming.
        trait_impl = create_trait_impl(5)
        self.assertEqual(trait_impl.renamed_trait_method(10), 50) # 10 * 5

    def test_binding_renames(self):
        # Test Python-specific renames for "binding" items
        record = PyRecord(python_item=42)
        self.assertEqual(record.python_item, 42)

        # Test renamed enum
        enum1 = PyEnum.PYTHON_VARIANT_A()
        enum2 = PyEnum.PYTHON_RECORD(record)
        self.assertEqual(enum2[0].python_item, 42)

        # Test renamed enum fields
        enum1 = PyEnumWithFields.PYTHON_VARIANT_A(python_int = 1)
        self.assertEqual(enum1.python_int, 1)
        enum2 = PyEnumWithFields.PYTHON_RECORD(python_record = record, python_int = 1)
        self.assertEqual(enum2.python_record, record)

        # Test renamed function
        result = py_function(python_record=record)
        self.assertEqual(result[0].python_item, 42)

        # Test renamed object with renamed method
        obj = PyObject(python_value = 5)
        self.assertEqual(obj.python_method(python_arg=10), 15)

        # Test renamed error type exists
        self.assertTrue(hasattr(PyError, 'PythonSimple'))

        # Test trait method renaming
        trait = create_binding_trait_impl(5)
        self.assertEqual(trait.python_trait_method(10), 50)

if __name__ == '__main__':
    unittest.main()
