import asyncio
from asynccalc import *

class AsyncCalculatorImpl(AsyncCalculator):
    def __init__(self):
        self.py_calculator = PyAsyncCalculator()

    def double(self, promise, value):
        asyncio.create_task(self.py_calculator.double(value)).add_done_callback(lambda fut: promise.resolve(fut.result()))

    def square(self, promise, value):
        asyncio.create_task(self.py_calculator.square(value)).add_done_callback(lambda fut: promise.resolve(fut.result()))

class PromiseIntImpl(PromiseInt):
    def __init__(self):
        self.future = asyncio.Future()

    def resolve(self, value):
        self.future.set_result(value)
        print("Promise resolved: ", value)

async def double_then_square_wrapper(value):
    promise = PromiseIntImpl()
    double_then_square(promise, AsyncCalculatorImpl(), 4)
    return await promise.future

class PyAsyncCalculator:
    async def double(self, value):
        # Imagine awaiting some slow network request here
        return value + value

    async def square(self, value):
        # Imagine awaiting some CPU-bound process running in a different thread here
        return value * value

async def main():
    assert await double_then_square_wrapper(4) == 64

asyncio.run(main())
