import unittest
from callbacks import *

class CallAnswererImpl(CallAnswerer):
    def __init__(self, mode):
        self.mode = mode

    def answer(self):
        if self.mode == "ready":
            return "Bonjour"
        elif self.mode == "busy":
            raise TelephoneError.Busy()
        else:
            raise ValueError("Testing an unexpected error")

class CallbacksTest(unittest.TestCase):
    def test_answer(self):
        cb_object = CallAnswererImpl("ready")
        telephone = Telephone()
        self.assertEqual("Bonjour", telephone.call(cb_object))

    def test_busy(self):
        cb_object = CallAnswererImpl("busy")
        telephone = Telephone()
        with self.assertRaises(TelephoneError.Busy):
            telephone.call(cb_object)

    def test_unexpected_error(self):
        cb_object = CallAnswererImpl("something-else")
        telephone = Telephone()
        with self.assertRaises(TelephoneError.InternalTelephoneError):
            telephone.call(cb_object)

unittest.main()
