import asyncio
from uniffi_example_futures import *

async def main():
    assert(await say_after(20, 'Alice') == "Hello, Alice!")

if __name__ == '__main__':
    asyncio.run(main())
