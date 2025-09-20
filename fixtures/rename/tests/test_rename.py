# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
from uniffi_fixture_rename import *

class TestRename(unittest.TestCase):
    def test_rename(self):
        # Test renamed record
        record = RenamedRecord(item=42)
        self.assertEqual(record.item, 42)

        # Test renamed enum
        enum1 = RenamedEnum.VARIANT_A()
        enum2 = RenamedEnum.RECORD(record)
        self.assertEqual(enum2[0].item, 42)

        # Test renamed function
        result = renamed_function(record)
        self.assertEqual(result[0].item, 42)

        # Test renamed object with renamed constructor and method
        obj = RenamedObject.renamed_constructor(123)
        self.assertIsInstance(obj, RenamedObject)
        self.assertEqual(obj.renamed_method(), 123)

        # Test renamed error type exists
        self.assertTrue(hasattr(RenamedError, 'Simple'))

        # Test trait method renaming.
        trait_impl = create_trait_impl(5)
        self.assertEqual(trait_impl.renamed_trait_method(10), 50) # 10 * 5

if __name__ == '__main__':
    unittest.main()
