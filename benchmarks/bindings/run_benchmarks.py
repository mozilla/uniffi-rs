# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from benchmarks import *
import time

# Create objects to use in the tests.  This way the benchmarks don't include the time needed to
# construct these objects.

TEST_LARGE_STRING_1 = "a" * 2048
TEST_LARGE_STRING_2 = "b" * 1500
TEST_REC1 = TestRecord(a=-1, b=1, c=1.5)
TEST_REC2 = TestRecord(a=-2, b=2, c=4.5)
TEST_ENUM1 = TestEnum.ONE(a=-1, b=0)
TEST_ENUM2 = TestEnum.TWO(c=1.5)
TEST_VEC1 = [0, 1]
TEST_VEC2 = [2, 4, 6]
TEST_MAP1 = { 0: 1, 1: 2 }
TEST_MAP2 = { 2: 4 }
TEST_INTERFACE = TestInterface()
TEST_INTERFACE2 = TestInterface()
TEST_TRAIT_INTERFACE = make_test_trait_interface()
TEST_TRAIT_INTERFACE2 = make_test_trait_interface()
TEST_NESTED_DATA1 = NestedData(
    a=[TestRecord(a=-1, b=1, c=1.5)],
    b=[["one", "two"], ["three"]],
    c={
        "one": TestEnum.ONE(a=-1, b=1),
        "two": TestEnum.TWO(c=0.5),
    },
)
TEST_NESTED_DATA2 = NestedData(
    a=[TestRecord(a=-2, b=2, c=4.5)],
    b=[["four", "five"]],
    c={
        "two": TestEnum.TWO(c=-0.5),
    },
)

class TestCallbackObj(TestCallbackInterface):
    def call_only(self):
        pass

    def primitives(self, a, b):
        return a + b

    def strings(self, a, b):
        return a + b

    def large_strings(self, a, b):
        return a + b

    def records(self, a, b):
        return TestRecord(
            a=a.a + b.a,
            b=a.b + b.b,
            c=a.c + b.c,
        )

    def enums(self, a, b):
        if isinstance(a, TestEnum.ONE):
            a_sum = a.a + a.b
        else:
            a_sum = a.c
        if isinstance(b, TestEnum.ONE):
            b_sum = b.a + b.b
        else:
            b_sum = b.c
        return TestEnum.TWO(a_sum + b_sum)

    def vecs(self, a, b):
        return a + b

    def hash_maps(self, a, b):
        return a | b

    def interfaces(self, a, b):
        # Perform some silliness to make sure Python needs to access both `a` and `b`
        if a == b:
            return a
        else:
            return b

    def trait_interfaces(self, a, b):
        # Perform some silliness to make sure Python needs to access both `a` and `b`
        if a == b:
            return a
        else:
            return b

    def nested_data(self, a, b):
        return NestedData(a=a.a + b.a, b=a.b + b.b, c=a.c | b.c)

    def errors(self):
        raise TestError.Two

    def run_test(self, test_case, count):
        if test_case == TestCase.CALL_ONLY:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_call_only()
        elif test_case == TestCase.PRIMITIVES:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_primitives(0, -1)
        elif test_case == TestCase.STRINGS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_strings("a", "b")
        elif test_case == TestCase.LARGE_STRINGS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_large_strings(
                    TEST_LARGE_STRING_1,
                    TEST_LARGE_STRING_2
                )
        elif test_case == TestCase.RECORDS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_records(TEST_REC1, TEST_REC2)
        elif test_case == TestCase.ENUMS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_enums(TEST_ENUM1, TEST_ENUM2)
        elif test_case == TestCase.VECS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_vecs(TEST_VEC1, TEST_VEC2)
        elif test_case == TestCase.HASHMAPS:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_hashmaps(TEST_MAP1, TEST_MAP2)
        elif test_case == TestCase.INTERFACES:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_interfaces(TEST_INTERFACE, TEST_INTERFACE2)
        elif test_case == TestCase.TRAIT_INTERFACES:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_trait_interfaces(TEST_TRAIT_INTERFACE, TEST_TRAIT_INTERFACE2)
        elif test_case == TestCase.NESTED_DATA:
            start = time.perf_counter_ns()
            for _ in range(count):
                test_case_nested_data(TEST_NESTED_DATA1, TEST_NESTED_DATA2)
        elif test_case == TestCase.ERRORS:
            start = time.perf_counter_ns()
            for _ in range(count):
                try:
                    test_case_errors()
                except:
                    pass

        end = time.perf_counter_ns()
        return end - start

run_benchmarks("python", TestCallbackObj())
