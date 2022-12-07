import uniffi_futures
import Foundation // To get `DispatchGroup` and `Date` types.

var counter = DispatchGroup()

// Test `alwaysReady`
counter.enter()

Task {
	let t0 = Date()
	let result = await alwaysReady()
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration < 0.1)
	assert(result == true)

	counter.leave()
}

// Test `Sleep`
counter.enter()

Task {
	let t0 = Date()
	let result = await sleep(secs: 2)
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 2 && tDelta.duration < 2.1)
	assert(result == true)

	counter.leave()
}

// Test sequential futures.
counter.enter()

Task {
	let t0 = Date()
	let result_alice = await sayAfter(secs: 1, who: "Alice")
	let result_bob = await sayAfter(secs: 2, who: "Bob")
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 3 && tDelta.duration < 3.1)
	assert(result_alice == "Hello, Alice!")
	assert(result_bob == "Hello, Bob!")

	counter.leave()
}

// Test concurrent futures.
counter.enter()

Task {
	async let alice = sayAfter(secs: 1, who: "Alice")
	async let bob = sayAfter(secs: 2, who: "Bob")

	let t0 = Date()
	let (result_alice, result_bob) = await (alice, bob)
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 2 && tDelta.duration < 2.1)
	assert(result_alice == "Hello, Alice!")
	assert(result_bob == "Hello, Bob!")

	counter.leave()
}

// Test async methods
counter.enter()

Task {
	let megaphone = newMegaphone()

	let t0 = Date()
	let result_alice = await megaphone.sayAfter(secs: 2, who: "Alice")
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 2 && tDelta.duration < 2.1)
	assert(result_alice == "HELLO, ALICE!")

	counter.leave()
}

counter.wait()
