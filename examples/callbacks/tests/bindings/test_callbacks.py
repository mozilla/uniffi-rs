import unittest
from callbacks import *

# This is defined in UDL as a "callback". It's not possible to have a Rust
# implementation of a callback, they only exist on the foreign side.
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

# This is a normal Rust trait - very much like a callback but can be implemented
# in Rust or in foreign code and is generally more consistent with the uniffi
# Arc<>-based object model.
class DiscountSim(SimCard):
    def name(self):
        return "python"

class CallbacksTest(unittest.TestCase):
    TelephoneImpl = Telephone
    def test_answer(self):
        cb_object = CallAnswererImpl("ready")
        telephone = self.TelephoneImpl()
        self.assertEqual("Bonjour", telephone.call(get_sim_cards()[0], cb_object))

    def test_busy(self):
        cb_object = CallAnswererImpl("busy")
        telephone = self.TelephoneImpl()
        with self.assertRaises(TelephoneError.Busy):
            telephone.call(get_sim_cards()[0], cb_object)

    def test_unexpected_error(self):
        cb_object = CallAnswererImpl("something-else")
        telephone = self.TelephoneImpl()
        with self.assertRaises(TelephoneError.InternalTelephoneError):
            telephone.call(get_sim_cards()[0], cb_object)

    def test_sims(self):
        cb_object = CallAnswererImpl("ready")
        telephone = self.TelephoneImpl()
        sim = DiscountSim()
        self.assertEqual("python est bon marché", telephone.call(sim, cb_object))

class FancyCallbacksTest(CallbacksTest):
    TelephoneImpl = FancyTelephone

unittest.main()
