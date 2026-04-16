from test_package.uniffi_bindgen_tests import *
import unittest

class TestErrors(unittest.TestCase):
    def test_interfaces(self):
        interface = TestInterface(20)
        self.assertEqual(interface.get_value(), 20)
        self.assertEqual(clone_interface(interface).get_value(), 20)

    def test_records_with_interface_fields(self):
        two = TwoTestInterfaces(first=TestInterface(1), second=TestInterface(2))
        swapped = swap_test_interfaces(two)
        self.assertEqual(swapped.first.get_value(), 2)
        self.assertEqual(swapped.second.get_value(), 1)

    def test_interface_refcounts(self):
        interface = TestInterface(20)
        interface2 = TestInterface(20)
        def func_that_clones_interface(interface):
          return clone_interface(interface)

        interface2_clone = func_that_clones_interface(clone_interface(interface))
        _ = interface.get_value()
        # Check that only the 2 actual references remain after the dust clears
        self.assertEqual(interface.ref_count(), 2)

if __name__ == '__main__':
    unittest.main()
