import asyncio
from uniffi_example_futures import *

async def main():
    assert(await say_after(20, 'Alice') == "Hello, Alice!")
    store = Store(asyncio.get_running_loop())
    assert(await store.load_item() == "this was loaded from disk")

if __name__ == '__main__':
    asyncio.run(main())
