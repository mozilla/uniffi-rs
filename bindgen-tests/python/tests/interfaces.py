from test_package.uniffi_bindgen_tests import *
import unittest

class TestErrors(unittest.TestCase):
    def test_interfaces(self):
        interface = TestInterface(20)
        self.assertEqual(interface.get_value(), 20)
        self.assertEqual(clone_interface(interface).get_value(), 20)

    def test_secondary_constructor(self):
        interface = TestInterface.secondary_constructor(20)
        self.assertEqual(interface.get_value(), 40)

    def test_records_with_interface_fields(self):
        two = TwoTestInterfaces(first=TestInterface(1), second=TestInterface(2))
        swapped = swap_test_interfaces(two)
        self.assertEqual(swapped.first.get_value(), 2)
        self.assertEqual(swapped.second.get_value(), 1)

    def test_enums_with_interfaces(self):
        en = TestInterfaceEnum.ONE(TestInterface(1))
        self.assertEqual(en.i.get_value(), 1)

    def test_interface_refcounts(self):
        interface = TestInterface(20)
        interface2 = TestInterface(20)
        def func_that_clones_interface(interface):
          return clone_interface(interface)

        interface2_clone = func_that_clones_interface(clone_interface(interface))
        _ = interface.get_value()
        # Check that only the 2 actual references remain after the dust clears
        self.assertEqual(interface.ref_count(), 2)

    def test_argument_name_mapping(self):
        interface = TestInterface(20)
        # the following calls will fail if the argument name differs
        interface.method_with_multi_word_arg(the_argument="test")

if __name__ == '__main__':
    unittest.main()
