from uniffi_futures import always_ready, void, sleep, say_after, new_megaphone, say_after_with_tokio, fallible_me, MyError, MyRecord, new_my_record
import unittest
from datetime import datetime
import asyncio

def now():
    return datetime.now()

class TestFutures(unittest.TestCase):
    def test_always_ready(self):
        async def test():
            t0 = now()
            result = await always_ready()
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta < 0.1)
            self.assertEqual(result, True)

        asyncio.run(test())

    def test_void(self):
        async def test():
            t0 = now()
            result = await void()
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta < 0.1)
            self.assertEqual(result, None)

        asyncio.run(test())

    def test_sleep(self):
        async def test():
            t0 = now()
            await sleep(2000)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 2 and t_delta < 2.1)

        asyncio.run(test())

    def test_sequential_futures(self):
        async def test():
            t0 = now()
            result_alice = await say_after(100, 'Alice')
            result_bob = await say_after(200, 'Bob')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 0.3 and t_delta < 0.31)
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
            self.assertTrue(t_delta > 0.2 and t_delta < 0.21)
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
            self.assertTrue(t_delta > 0.2 and t_delta < 0.21)
            self.assertEqual(result_alice, 'HELLO, ALICE!')

        asyncio.run(test())

    def test_with_tokio_runtime(self):
        async def test():
            t0 = now()
            result_alice = await say_after_with_tokio(200, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 0.2 and t_delta < 0.21)
            self.assertEqual(result_alice, 'Hello, Alice (with Tokio)!')

        asyncio.run(test())

    def test_fallible(self):
        async def test():
            t0 = now()
            result = await fallible_me(False)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 0 and t_delta < 0.1)
            self.assertEqual(result, 42)

            try:
                result = await fallible_me(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                self.assertTrue(True)

            megaphone = new_megaphone()

            t0 = now()
            result = await megaphone.fallible_me(False)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 0 and t_delta < 0.1)
            self.assertEqual(result, 42)

            try:
                result = await megaphone.fallible_me(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                self.assertTrue(True)

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

if __name__ == '__main__':
    unittest.main()
