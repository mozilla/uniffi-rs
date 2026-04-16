from test_package.uniffi_bindgen_tests import *
import unittest

class TestFutures(unittest.IsolatedAsyncioTestCase):
    async def test_simple_calls(self):
        self.assertEqual(await async_roundtrip_u8(42), 42)
        self.assertEqual(await async_roundtrip_i8(-42), -42)
        self.assertEqual(await async_roundtrip_u16(42), 42)
        self.assertEqual(await async_roundtrip_i16(-42), -42)
        self.assertEqual(await async_roundtrip_u32(42), 42)
        self.assertEqual(await async_roundtrip_i32(-42), -42)
        self.assertEqual(await async_roundtrip_u64(42), 42)
        self.assertEqual(await async_roundtrip_i64(-42), -42)
        self.assertEqual(await async_roundtrip_f32(0.5), 0.5)
        self.assertEqual(await async_roundtrip_f64(-0.5), -0.5)
        self.assertEqual(await async_roundtrip_string("hi"), "hi")
        self.assertEqual(await async_roundtrip_vec([42]), [42])
        self.assertEqual(await async_roundtrip_map({ "hello": "world" }), { "hello": "world" })

    async def test_errors(self):
        with self.assertRaises(TestError.Failure1):
            await async_throw_error()

    async def test_methods(self):
        obj = AsyncInterface("Alice")
        self.assertEqual(await obj.name(), "Alice")

        obj2 = await async_roundtrip_obj(obj)
        self.assertEqual(await obj2.name(), "Alice")

    async def test_async_callback_interfaces(self):
        class AsyncCallbackImpl:
            ref_count = 0

            def __init__(self, value):
                self.value = value
                AsyncCallbackImpl.ref_count += 1

            def __del__(self):
                AsyncCallbackImpl.ref_count -= 1

            async def noop(self):
                pass

            async def get_value(self):
                return self.value

            async def set_value(self, value):
                self.value = value

            async def throw_if_equal(self, numbers):
                if numbers.a == numbers.b:
                    raise TestError.Failure1()
                return numbers

        cbi = AsyncCallbackImpl(42)
        await invoke_test_async_callback_interface_noop(cbi)

        self.assertEqual(
            await invoke_test_async_callback_interface_get_value(cbi),
            42,
        )

        await invoke_test_async_callback_interface_set_value(cbi, 43)
        self.assertEqual(await invoke_test_async_callback_interface_get_value(cbi), 43)

        with self.assertRaises(TestError.Failure1):
           await invoke_test_async_callback_interface_throw_if_equal(
                cbi,
                CallbackInterfaceNumbers(a=10, b=10)
            )

        self.assertEqual(
            await invoke_test_async_callback_interface_throw_if_equal(
                cbi,
                CallbackInterfaceNumbers(a=10, b=11)
            ),
            CallbackInterfaceNumbers(a=10, b=11)
        )

        # The previcalls created a bunch of callback interface references.  Make sure they've been cleaned
        # up and the only remaining reference is for our `cbi` variable.
        self.assertEqual(AsyncCallbackImpl.ref_count, 1)

if __name__ == '__main__':
    unittest.main()
