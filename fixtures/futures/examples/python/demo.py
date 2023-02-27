from uniffi_futures import sleep, say_after, void
import asyncio
import time

def show_time():
    print(f"[time {time.strftime('%X')}]")

async def main():
    print("Let's start!\n")

    print('Wait 2secs before greeting you, dear public!\n')

    show_time()
    result = await say_after(2000, 'You')
    print(f'result: {result}')
    show_time()

    print("\nWouah, 'tired. Let's sleep for 3secs!\n")

    show_time()
    await sleep(3000)
    show_time()

    print("\nIs it really blocking? Nah. Let's greet Alice and Bob after resp. 2secs and 3secs _concurrently_!\n")

    alice = asyncio.create_task(say_after(2000, 'Alice'))
    bob = asyncio.create_task(say_after(3000, 'Bob'))
    show_time()
    result_alice = await alice
    result_bob = await bob
    print(f'result_alice: {result_alice}')
    print(f'result_bob: {result_bob}')

    show_time()
    print("\nSee, it took 3secs, not 5secs!")

asyncio.run(main())
