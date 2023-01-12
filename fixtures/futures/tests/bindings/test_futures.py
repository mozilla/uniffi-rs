from uniffi_futures import always_ready, void, sleep, say_after, new_megaphone, say_after_with_tokio, fallible_me, MyError
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
            await sleep(2)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 2 and t_delta < 2.1)

        asyncio.run(test())

    def test_sequential_futures(self):
        async def test():
            t0 = now()
            result_alice = await say_after(1, 'Alice')
            result_bob = await say_after(2, 'Bob')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 3 and t_delta < 3.1)
            self.assertEqual(result_alice, 'Hello, Alice!')
            self.assertEqual(result_bob, 'Hello, Bob!')

        asyncio.run(test())

    def test_concurrent_tasks(self):
        async def test():
            alice = asyncio.create_task(say_after(1, 'Alice'))
            bob = asyncio.create_task(say_after(2, 'Bob'))

            t0 = now()
            result_alice = await alice
            result_bob = await bob
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 2 and t_delta < 2.1)
            self.assertEqual(result_alice, 'Hello, Alice!')
            self.assertEqual(result_bob, 'Hello, Bob!')

        asyncio.run(test())

    def test_async_methods(self):
        async def test():
            megaphone = new_megaphone()
            t0 = now()
            result_alice = await megaphone.say_after(2, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 2 and t_delta < 2.1)
            self.assertEqual(result_alice, 'HELLO, ALICE!')

        asyncio.run(test())

    def test_with_tokio_runtime(self):
        async def test():
            t0 = now()
            result_alice = await say_after_with_tokio(2, 'Alice')
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 2 and t_delta < 2.1)
            self.assertEqual(result_alice, 'Hello, Alice (with Tokio)!')

        asyncio.run(test())

    def test_fallible(self):
        async def test():
            t0 = now()
            result = await fallible_me(False)
            t1 = now()

            t_delta = (t1 - t0).total_seconds()
            self.assertTrue(t_delta > 0 and t_delta < 0.1)
            self.assertEqual(result.value, 42)

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
            self.assertEqual(result.value, 42)

            try:
                result = await megaphone.fallible_me(True)
                self.assertTrue(False) # should never be reached
            except MyError as exception:
                self.assertTrue(True)

        asyncio.run(test())

if __name__ == '__main__':
    unittest.main()