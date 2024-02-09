from futures import *
import unittest
from datetime import datetime
import asyncio
import typing

def now():
    return datetime.now()

class TestFutures(unittest.TestCase):
    def test_always_ready(self):
        async def test():
            self.assertEqual(await always_ready(), True)

        asyncio.run(test())

    def test_void(self):
        async def test():
            self.assertEqual(await void(), None)

        asyncio.run(test())

    def test_sleep(self):
        async def test():
            t0 = now()
            await sleep(2000)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 2)

        asyncio.run(test())

    def test_sequential_futures(self):
        async def test():
            t0 = now()
            result_alice = await say_after(100, 'Alice')
            result_bob = await say_after(200, 'Bob')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.3)
            self.assertEqual(result_alice, 'Hello, Alice!')
            self.assertEqual(result_bob, 'Hello, Bob!')

        asyncio.run(test())

    def test_concurrent_tasks(self):
        async def test():
            alice = asyncio.create_task(say_after(100, 'Alice'))
            bob = asyncio.create_task(say_after(200, 'Bob'))

            t0 = now()
            result_alice = await alice
            result_bob = await bob
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)
            self.assertEqual(result_alice, 'Hello, Alice!')
            self.assertEqual(result_bob, 'Hello, Bob!')

        asyncio.run(test())

    def test_async_methods(self):
        async def test():
            megaphone = new_megaphone()
            t0 = now()
            result_alice = await megaphone.say_after(200, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)
            self.assertEqual(result_alice, 'HELLO, ALICE!')

        asyncio.run(test())

    def test_async_trait_interface_methods(self):
        async def test():
            traits = get_say_after_traits()
            t0 = now()
            result1 = await traits[0].say_after(100, 'Alice')
            result2 = await traits[1].say_after(100, 'Bob')
            t1 = now()

            self.assertEqual(result1, 'Hello, Alice!')
            self.assertEqual(result2, 'Hello, Bob!')
            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)

        asyncio.run(test())

    def test_udl_async_trait_interface_methods(self):
        async def test():
            traits = get_say_after_udl_traits()
            t0 = now()
            result1 = await traits[0].say_after(100, 'Alice')
            result2 = await traits[1].say_after(100, 'Bob')
            t1 = now()

            self.assertEqual(result1, 'Hello, Alice!')
            self.assertEqual(result2, 'Hello, Bob!')
            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)

        asyncio.run(test())

    def test_async_object_param(self):
        async def test():
            megaphone = new_megaphone()
            t0 = now()
            result_alice = await say_after_with_megaphone(megaphone, 200, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)
            self.assertEqual(result_alice, 'HELLO, ALICE!')

        asyncio.run(test())

    def test_with_tokio_runtime(self):
        async def test():
            t0 = now()
            result_alice = await say_after_with_tokio(200, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertGreater(t_delta, 0.2)
            self.assertEqual(result_alice, 'Hello, Alice (with Tokio)!')

        asyncio.run(test())

    def test_fallible(self):
        async def test():
            result = await fallible_me(False)
            self.assertEqual(result, 42)

            try:
                result = await fallible_me(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                self.assertTrue(True)

            megaphone = new_megaphone()

            result = await megaphone.fallible_me(False)
            self.assertEqual(result, 42)

            try:
                result = await megaphone.fallible_me(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                self.assertTrue(True)

        asyncio.run(test())

    def test_fallible_struct(self):
        async def test():
            megaphone = await fallible_struct(False)
            self.assertEqual(await megaphone.fallible_me(False), 42)

            try:
                await fallible_struct(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                pass

        asyncio.run(test())

    def test_record(self):
        async def test():
            result = await new_my_record("foo", 42)
            self.assertEqual(result.__class__, MyRecord)
            self.assertEqual(result.a, "foo")
            self.assertEqual(result.b, 42)

        asyncio.run(test())

    def test_cancel(self):
        async def test():
            # Create a task
            task = asyncio.create_task(say_after(200, 'Alice'))
            # Wait to ensure that the polling has started, then cancel the task
            await asyncio.sleep(0.1)
            task.cancel()
            # Wait long enough for the Rust callback to fire.  This shouldn't cause an exception,
            # even though the task is cancelled.
            await asyncio.sleep(0.2)
            # Awaiting the task should result in a CancelledError.
            with self.assertRaises(asyncio.CancelledError):
                await task

        asyncio.run(test())

    # Test a future that uses a lock and that is cancelled.
    def test_shared_resource_cancellation(self):
        async def test():
            task = asyncio.create_task(use_shared_resource(
                SharedResourceOptions(release_after_ms=5000, timeout_ms=100)))
            # Wait some time to ensure the task has locked the shared resource
            await asyncio.sleep(0.05)
            task.cancel()
            await use_shared_resource(SharedResourceOptions(release_after_ms=0, timeout_ms=1000))
        asyncio.run(test())

    def test_shared_resource_no_cancellation(self):
        async def test():
            await use_shared_resource(SharedResourceOptions(release_after_ms=100, timeout_ms=1000))
            await use_shared_resource(SharedResourceOptions(release_after_ms=0, timeout_ms=1000))
        asyncio.run(test())

    def test_function_annotations(self):
        async def test():
            self.assertEqual(typing.get_type_hints(sleep) , {"ms": int, "return": bool})
            self.assertEqual(typing.get_type_hints(sleep_no_return), {"ms": int, "return": type(None)})
        asyncio.run(test())

if __name__ == '__main__':
    unittest.main()
