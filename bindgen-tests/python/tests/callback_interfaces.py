from test_package.uniffi_bindgen_tests import *
import threading
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

    def echo(self, s):
        return s

# Calls that re-enter Rust from inside a callback, so several calls are in progress on the same
# thread at once. An inner call's failure must not become the result of the call it's nested in,
# and the outer call's own result must still come through after the inner calls.
class ReentrantCallback(TestReentrantCallback):
    def reenter(self, depth):
        # A nested call that fails, caught here. The call this is nested in must still succeed.
        try:
            func_with_error(0)
            raise RuntimeError("Expected TestError.Failure1 to be raised")
        except TestError.Failure1:
            pass
        if depth == 0:
            return 0
        return invoke_test_reentrant_callback(self, depth - 1, False)

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
        assert(invoke_test_callback_interface_echo(cbi, "test-string") == "test-string")

        # The previous calls created a bunch of callback interface references.  Make sure they've been cleaned
        # up and the only remaining reference is for our `cbi` variable.
        assert(CallbackImpl.callback_ref_count == 1)

    def test_optional_callback_interfaces(self):
        # Callback interfaces can only be passed into Rust, so the Rust side calls a method on the
        # callback interface and returns `None` when it wasn't passed one.
        self.assertIsNone(invoke_optional_test_callback_interface_get_value(None))

        cbi = CallbackImpl(42)
        self.assertEqual(invoke_optional_test_callback_interface_get_value(cbi), 42)

        # Make sure the calls above didn't leak any callback interface references
        del cbi
        self.assertEqual(CallbackImpl.callback_ref_count, 0)

    def test_reentrant_calls(self):
        reentrant = ReentrantCallback()
        self.assertEqual(invoke_test_reentrant_callback(reentrant, 0, False), 1)
        # 20 levels of Rust -> Python -> Rust nesting, each with a failed call nested inside it.
        self.assertEqual(invoke_test_reentrant_callback(reentrant, 20, False), 21)
        # The outer call fails after the nested calls (including the failed ones) have finished,
        # and must report its own error.
        with self.assertRaises(TestError.Failure3) as cm:
            invoke_test_reentrant_callback(reentrant, 5, True)
        self.assertEqual(cm.exception[0], 5)
        # The same thread keeps working after the failures above.
        self.assertEqual(invoke_test_reentrant_callback(reentrant, 3, False), 4)

    def test_reentrant_calls_from_threads(self):
        # Several threads making reentrant calls, failing and succeeding, at the same time.
        reentrant = ReentrantCallback()
        failures = []

        def reentrant_calls(t):
            try:
                for i in range(200):
                    depth = (t + i) % 4
                    if i % 3 == 0:
                        with self.assertRaises(TestError.Failure3) as cm:
                            invoke_test_reentrant_callback(reentrant, depth, True)
                        self.assertEqual(cm.exception[0], depth)
                    else:
                        self.assertEqual(invoke_test_reentrant_callback(reentrant, depth, False), depth + 1)
            except Exception as e:
                failures.append(e)

        threads = [threading.Thread(target=reentrant_calls, args=(t,)) for t in range(8)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        self.assertEqual(failures, [])

if __name__ == '__main__':
    unittest.main()
