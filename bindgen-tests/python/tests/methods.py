from test_package.uniffi_bindgen_tests import *
import unittest

ARGS = "2 3 four five rec(6) rec(7) B A(8,9) B(ten,11) interface(12) interface(13)"

def describe_call(receiver):
    return receiver.describe_call(
        2,
        3,
        "four",
        "five",
        SimpleRec(a=6),
        SimpleRec(a=7),
        EnumNoData.B,
        EnumWithData.A(8, 9),
        EnumWithData.B("ten", 11),
        TestInterface(12),
        TestInterface(13),
    )

class TestMethods(unittest.TestCase):
    def test_record(self):
        self.assertEqual(describe_call(MethodTestRec(value=1)), f"rec(1) {ARGS}")

    def test_enum_no_data(self):
        self.assertEqual(describe_call(MethodTestEnumNoData.B), f"enum_no_data(B) {ARGS}")

    def test_enum_with_data(self):
        self.assertEqual(describe_call(MethodTestEnumWithData.A(1)), f"enum_with_data(A(1)) {ARGS}")
        self.assertEqual(describe_call(MethodTestEnumWithData.B("one")), f"enum_with_data(B(one)) {ARGS}")

    def test_interface(self):
        self.assertEqual(describe_call(MethodTestInterface(1)), f"interface(1) {ARGS}")

if __name__ == "__main__":
    unittest.main()
