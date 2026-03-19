from test_package.uniffi_bindgen_tests import *
import unittest


class TraitImpl(TestTraitInterface):
    ref_count = 0

    def __init__(self, value):
        self.value = value
        TraitImpl.ref_count += 1

    def __del__(self):
        TraitImpl.ref_count -= 1

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

class TestTraitInterfaces(unittest.TestCase):
    def check_rust_impl(self, rust_trait_impl):
        rust_trait_impl.noop()
        self.assertEqual(rust_trait_impl.get_value(), 42)
        rust_trait_impl.set_value(43)
        self.assertEqual(rust_trait_impl.get_value(), 43)
        with self.assertRaises(TestError.Failure1):
            rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers(a=10, b=10))
        self.assertEqual(
            rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers(a=10, b=11)),
            CallbackInterfaceNumbers(a=10, b=11))

    def check_py_impl(self, py_trait_impl):
        invoke_test_trait_interface_noop(py_trait_impl)
        self.assertEqual(invoke_test_trait_interface_get_value(py_trait_impl), 42)
        invoke_test_trait_interface_set_value(py_trait_impl, 43)
        self.assertEqual(invoke_test_trait_interface_get_value(py_trait_impl), 43)
        with self.assertRaises(TestError.Failure1):
            invoke_test_trait_interface_throw_if_equal(
                py_trait_impl,
                CallbackInterfaceNumbers(a=10, b=10)
            )

        self.assertEqual(
            invoke_test_trait_interface_throw_if_equal(
                py_trait_impl,
                CallbackInterfaceNumbers(a=10, b=11)
            ),
            CallbackInterfaceNumbers(a=10, b=11)
        )

    def test_rust_impl(self):
        self.check_rust_impl(create_test_trait_interface(42))

    def test_rust_impl_roundtripped(self):
        impl = roundtrip_test_trait_interface(create_test_trait_interface(42))
        self.check_rust_impl(impl)

    def test_rust_impl_roundtripped_list(self):
        impl = roundtrip_test_trait_interface_list([create_test_trait_interface(42)])[0]
        self.check_rust_impl(impl)

    def test_py_impl(self):
        self.check_py_impl(TraitImpl(42))
        self.assertEqual(TraitImpl.ref_count, 0)

    def test_py_impl_roundtripped(self):
        impl = roundtrip_test_trait_interface(TraitImpl(42))
        self.check_py_impl(impl)
        del impl
        self.assertEqual(TraitImpl.ref_count, 0)

    def test_py_impl_roundtripped_list(self):
        impl = roundtrip_test_trait_interface_list([TraitImpl(42)])[0]
        self.check_py_impl(impl)
        del impl
        self.assertEqual(TraitImpl.ref_count, 0)

class AsyncTraitImpl(AsyncTestTraitInterface):
    ref_count = 0

    def __init__(self, value):
        self.value = value
        AsyncTraitImpl.ref_count += 1

    def __del__(self):
        AsyncTraitImpl.ref_count -= 1

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

class TestAsyncTraitInterfaces(unittest.IsolatedAsyncioTestCase):
    async def check_async_rust_impl(self, async_rust_trait_impl):
        await async_rust_trait_impl.noop()
        self.assertEqual(await async_rust_trait_impl.get_value(), 42)
        await async_rust_trait_impl.set_value(43)
        self.assertEqual(await async_rust_trait_impl.get_value(), 43)
        with self.assertRaises(TestError.Failure1):
            await async_rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers(a=10, b=10))
        self.assertEqual(
            await async_rust_trait_impl.throw_if_equal(CallbackInterfaceNumbers(a=10, b=11)),
            CallbackInterfaceNumbers(a=10, b=11))

    async def check_async_py_impl(self, async_py_trait_impl):
        await invoke_async_test_trait_interface_noop(async_py_trait_impl)
        self.assertEqual(await invoke_async_test_trait_interface_get_value(async_py_trait_impl), 42)
        await invoke_async_test_trait_interface_set_value(async_py_trait_impl, 43)
        self.assertEqual(await invoke_async_test_trait_interface_get_value(async_py_trait_impl), 43)
        with self.assertRaises(TestError.Failure1):
            await invoke_async_test_trait_interface_throw_if_equal(
                async_py_trait_impl,
                CallbackInterfaceNumbers(a=10, b=10)
            )

        self.assertEqual(
            await invoke_async_test_trait_interface_throw_if_equal(
                async_py_trait_impl,
                CallbackInterfaceNumbers(a=10, b=11)
            ),
            CallbackInterfaceNumbers(a=10, b=11)
        )

    async def test_async_rust_impl(self):
        await self.check_async_rust_impl(create_async_test_trait_interface(42))

    async def test_async_rust_impl_roundtripped(self):
        impl = roundtrip_async_test_trait_interface(create_async_test_trait_interface(42))
        await self.check_async_rust_impl(impl)

    async def test_async_rust_impl_roundtripped_list(self):
        impl = roundtrip_async_test_trait_interface_list([create_async_test_trait_interface(42)])[0]
        await self.check_async_rust_impl(impl)

    async def test_async_py_impl(self):
        await self.check_async_py_impl(AsyncTraitImpl(42))
        self.assertEqual(AsyncTraitImpl.ref_count, 0)

    async def test_async_py_impl_roundtripped(self):
        impl = roundtrip_async_test_trait_interface(AsyncTraitImpl(42))
        await self.check_async_py_impl(impl)
        del impl
        self.assertEqual(AsyncTraitImpl.ref_count, 0)

    async def test_async_py_impl_roundtripped_list(self):
        impl = roundtrip_async_test_trait_interface_list([AsyncTraitImpl(42)])[0]
        await self.check_async_py_impl(impl)
        del impl
        self.assertEqual(AsyncTraitImpl.ref_count, 0)

if __name__ == '__main__':
    unittest.main()
