# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import asyncio
import unittest
import weakref
from fixture_foreign_executor import ForeignExecutorTester

class TestForeignExecutor(unittest.TestCase):
    def test_schedule(self):
        async def run_test(constructor, delay):
            if constructor == "primary":
                tester = ForeignExecutorTester(asyncio.get_running_loop())
            elif constructor == "new_from_sequence":
                tester = ForeignExecutorTester.new_from_sequence([asyncio.get_running_loop()])
            else:
                raise AssertionError(f"Unknown constructor: {constructor}")
            tester.schedule_test(delay)
            await asyncio.sleep((delay / 1000) + 0.1)
            return tester.get_last_result()

        # Test no delay and lifting the foreign executor directly
        result = asyncio.run(run_test("primary", 0))
        self.assertTrue(result.call_happened_in_different_thread)
        self.assertTrue(result.delay_ms <= 1)

        # Test no delay and reading the foreign executor from a list
        result = asyncio.run(run_test("new_from_sequence", 10))
        self.assertTrue(result.call_happened_in_different_thread)
        self.assertTrue(9 <= result.delay_ms <= 11)

    def test_reference_counts(self):
        # Create an event loop
        loop = asyncio.new_event_loop()
        loop_ref = weakref.ref(loop)
        # Create ForeignExecutorTester that stores the loop
        tester = ForeignExecutorTester(loop)
        tester2 = ForeignExecutorTester.new_from_sequence([loop]),
        # Test that testers hold a reference to the loop.  After deleting the loop, the weakref should still be alive
        loop.close()
        del loop
        self.assertNotEqual(loop_ref(), None, "ForeignExecutor didn't take a reference to the event loop")
        # Deleting testers should cause the loop to be destroyed
        del tester
        del tester2
        self.assertEqual(loop_ref(), None, "ForeignExecutor didn't release a reference to the event loop")

if __name__=='__main__':
    unittest.main()
