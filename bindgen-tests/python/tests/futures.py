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

        rec = AsyncRecord(name="Bob")
        assert(await rec.get_name() == "Bob")

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

    async def test_boxed_future_callback_interface(self):
        # A callback interface whose async method is a hand-written boxed future (no
        # `#[async_trait]`); it should behave like any other async callback method.
        class BoxedFutureTraitImpl:
            async def reply(self, ms, who):
                return f"{who} replied at {ms}"

        cbi = BoxedFutureTraitImpl()
        self.assertEqual(
            await invoke_boxed_future_trait(cbi, 1234, "Alice"),
            "Alice replied at 1234",
        )

# Async callback interfaces with fallible methods that return non-record values
class AsyncFallibleCallbackImpl(TestAsyncFallibleCallbackInterface):
    async def get_u64(self, fail):
        if fail:
            raise TestError.Failure1()
        return 42

    async def get_bool(self, fail):
        if fail:
            raise TestError.Failure1()
        return True

    async def get_string(self, fail):
        if fail:
            raise TestError.Failure1()
        return "hello"

    async def get_optional_string(self, fail):
        if fail:
            raise TestError.Failure1()
        return None

class TestAsyncFallibleCallbackInterfaces(unittest.IsolatedAsyncioTestCase):
    async def test_success(self):
        cbi = AsyncFallibleCallbackImpl()
        self.assertEqual(
            await invoke_test_async_fallible_callback_interface_get_u64(cbi, False),
            42,
        )
        self.assertEqual(
            await invoke_test_async_fallible_callback_interface_get_bool(cbi, False),
            True,
        )
        self.assertEqual(
            await invoke_test_async_fallible_callback_interface_get_string(cbi, False),
            "hello",
        )
        self.assertEqual(
            await invoke_test_async_fallible_callback_interface_get_optional_string(cbi, False),
            None,
        )

    async def test_error(self):
        cbi = AsyncFallibleCallbackImpl()
        with self.assertRaises(TestError.Failure1):
            await invoke_test_async_fallible_callback_interface_get_u64(cbi, True)
        with self.assertRaises(TestError.Failure1):
            await invoke_test_async_fallible_callback_interface_get_bool(cbi, True)
        with self.assertRaises(TestError.Failure1):
            await invoke_test_async_fallible_callback_interface_get_string(cbi, True)
        with self.assertRaises(TestError.Failure1):
            await invoke_test_async_fallible_callback_interface_get_optional_string(cbi, True)

if __name__ == '__main__':
    unittest.main()
