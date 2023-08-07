import futures
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

// Test record.
counter.enter()

Task {
	let result = await newMyRecord(a: "foo", b: 42)

	assert(result.a == "foo")
	assert(result.b == 42)

	counter.leave()
}

// Test `void`
counter.enter()

Task {
	let t0 = Date()
	await void()
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration < 0.1)

	counter.leave()
}

// Test `Sleep`
counter.enter()

Task {
	let t0 = Date()
	let result = await sleep(ms: 2000)
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
	let result_alice = await sayAfter(ms: 1000, who: "Alice")
	let result_bob = await sayAfter(ms: 2000, who: "Bob")
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
	async let alice = sayAfter(ms: 1000, who: "Alice")
	async let bob = sayAfter(ms: 2000, who: "Bob")

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
	let result_alice = await megaphone.sayAfter(ms: 2000, who: "Alice")
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 2 && tDelta.duration < 2.1)
	assert(result_alice == "HELLO, ALICE!")

	counter.leave()
}

// Test async function returning an object
counter.enter()

Task {
	let megaphone = await asyncNewMegaphone()

	let result = try await megaphone.fallibleMe(doFail: false)
	assert(result == 42)

	counter.leave()
}

// Test with the Tokio runtime.
counter.enter()

Task {
	let t0 = Date()
	let result_alice = await sayAfterWithTokio(ms: 2000, who: "Alice")
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 2 && tDelta.duration < 2.1)
	assert(result_alice == "Hello, Alice (with Tokio)!")

	counter.leave()
}

// Test fallible function/method…
// … which doesn't throw.
counter.enter()

Task {
	let t0 = Date()
	let result = try await fallibleMe(doFail: false)
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 0 && tDelta.duration < 0.1)
	assert(result == 42)

	counter.leave()
}

Task {
	let m = try await fallibleStruct(doFail: false)
	let result = try await m.fallibleMe(doFail: false)
	assert(result == 42)
}

counter.enter()

Task {
	let megaphone = newMegaphone()

	let t0 = Date()
	let result = try await megaphone.fallibleMe(doFail: false)
	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 0 && tDelta.duration < 0.1)
	assert(result == 42)

	counter.leave()
}

// … which does throw.
counter.enter()

Task {
	let t0 = Date()

	do {
		let _ = try await fallibleMe(doFail: true)
	} catch MyError.Foo {
		assert(true)
	} catch {
		assert(false) // should never be reached
	}

	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 0 && tDelta.duration < 0.1)

	counter.leave()
}

Task {
	do {
		let _ = try await fallibleStruct(doFail: true)
	} catch MyError.Foo {
		assert(true)
	} catch {
		assert(false)
	}
}

counter.enter()

Task {
	let megaphone = newMegaphone()

	let t0 = Date()

	do {
		let _ = try await megaphone.fallibleMe(doFail: true)
	} catch MyError.Foo {
		assert(true)
	} catch {
		assert(false) // should never be reached
	}

	let t1 = Date()

	let tDelta = DateInterval(start: t0, end: t1)
	assert(tDelta.duration > 0 && tDelta.duration < 0.1)

	counter.leave()
}

// Test a future that uses a lock and that is cancelled.
counter.enter()
Task {
	let task = Task {
	    try! await useSharedResource(options: SharedResourceOptions(releaseAfterMs: 100, timeoutMs: 1000))
	}

	// Wait some time to ensure the task has locked the shared resource
	try await Task.sleep(nanoseconds: 50_000_000)
	// Cancel the job task the shared resource has been released.
	//
	// FIXME: this test currently passes because `test.cancel()` doesn't actually cancel the
	// operation.  We need to rework the Swift async handling to handle this properly.
	task.cancel()

	// Try accessing the shared resource again.  The initial task should release the shared resource
	// before the timeout expires.
	try! await useSharedResource(options: SharedResourceOptions(releaseAfterMs: 0, timeoutMs: 1000))
	counter.leave()
}

// Test a future that uses a lock and that is not cancelled.
counter.enter()
Task {
	try! await useSharedResource(options: SharedResourceOptions(releaseAfterMs: 100, timeoutMs: 1000))
	try! await useSharedResource(options: SharedResourceOptions(releaseAfterMs: 0, timeoutMs: 1000))
	counter.leave()
}

counter.wait()
