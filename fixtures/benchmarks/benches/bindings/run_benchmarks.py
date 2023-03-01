# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from benchmarks import *
import time

class TestCallbackObj:
    def test_method(self, a, b, data):
        return data.bar

    def test_void_return(self, a, b, data):
        pass

    def test_no_args_void_return(self):
        pass

    def run_test(self, test_case, count):
        data = TestData("StringOne", "StringTwo")
        if test_case == TestCase.FUNCTION:
            start = time.perf_counter_ns()
            for i in range(count):
                test_function(10, 20, data)
        elif test_case == TestCase.VOID_RETURN:
            start = time.perf_counter_ns()
            for i in range(count):
                test_void_return(10, 20, data)
        elif test_case == TestCase.NO_ARGS_VOID_RETURN:
            start = time.perf_counter_ns()
            for i in range(count):
                test_no_args_void_return()
        end = time.perf_counter_ns()
        return end - start

run_benchmarks("python", TestCallbackObj())
