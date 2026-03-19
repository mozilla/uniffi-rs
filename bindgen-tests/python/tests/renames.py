from test_package.uniffi_bindgen_tests import *
import unittest

class TestRenames(unittest.TestCase):
    def test_rename_record(self):
        rec = RenamedRecord(item=42)
        self.assertEqual(rec.item, 42)

    def test_rename_enum(self):
        rec = RenamedRecord(item=42)
        enum1 = RenamedEnum.RENAMED_VARIANT
        enum2 = RenamedEnum.RECORD(rec)
        return_value = renamed_function(rec)
        self.assertIsInstance(return_value, RenamedEnum.RECORD)
        self.assertEqual(return_value[0], rec)

    def test_rename_obj(self):
        obj = RenamedObject.renamed_constructor(123)
        self.assertEqual(obj.renamed_method(), 123)

    def test_trait_method(self):
        trait_impl = create_trait_impl(5)
        self.assertEqual(trait_impl.renamed_trait_method(10), 50)

    def test_py_rename_record(self):
        rec = PyRecord(py_item=100)
        self.assertEqual(rec.py_item, 100)

    def test_py_rename_enum(self):
        rec = PyRecord(py_item=100)
        PyEnum.PY_VARIANT_A()
        PyEnum.PY_RECORD(rec)
        PyEnumWithFields.PY_VARIANT_A(1)

    def test_py_rename_functions(self):
        rec = PyRecord(py_item=100)
        return_value = py_function(rec)
        self.assertIsInstance(return_value, PyEnum.PY_RECORD)
        self.assertEqual(return_value[0], rec)

        with self.assertRaises(PyError.PySimple):
           py_function(None)

    def test_py_rename_methods(self):
        obj = PyObject(200)
        self.assertEqual(obj.py_method(50), 250)

    def test_py_rename_trait(self):
        trait_impl = create_binding_trait_to_rename_impl(3)
        self.assertEqual(trait_impl.py_trait_method(7), 21)

if __name__ == '__main__':
    unittest.main()
