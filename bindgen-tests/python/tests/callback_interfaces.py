from test_package.uniffi_bindgen_tests import *
import unittest

class CallbackImpl(TestCallbackInterface):
    callback_ref_count = 0

    def __init__(self, value):
        self.value = value
        CallbackImpl.callback_ref_count += 1

    def __del__(self):
        CallbackImpl.callback_ref_count -= 1

    def noop(self):
        pass

    def get_value(self):
        return self.value

    def set_value(self, value):
        self.value = value

    def throw_if_equal(self, numbers):
        if numbers.a == numbers.b:
            raise TestError.Failure1()
        return numbers

class TestCallbackInterfaces(unittest.TestCase):
    def test_callback_interfaces(self):
        # Construct a callback interface to pass to rust
        cbi = CallbackImpl(42)
        # Test calling callback interface methods, which we can only do indirectly.
        # Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
        invoke_test_callback_interface_noop(cbi)
        assert(invoke_test_callback_interface_get_value(cbi) == 42)
        invoke_test_callback_interface_set_value(cbi, 43)
        assert(invoke_test_callback_interface_get_value(cbi) == 43)

        # The previous calls created a bunch of callback interface references.  Make sure they've been cleaned
        # up and the only remaining reference is for our `cbi` variable.
        assert(CallbackImpl.callback_ref_count == 1)

if __name__ == '__main__':
    unittest.main()
