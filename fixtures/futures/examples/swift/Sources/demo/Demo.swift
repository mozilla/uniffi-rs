import uniffi_futures
import Foundation // to get `Date` and `DateFormatter`

func showTime() {
	let now = Date()
	let formatter = DateFormatter()
	formatter.timeStyle = .medium
	print("[time \(formatter.string(from: now))]")
}

@main
struct Testing {
	static func main() async throws {
		print("Let's start!\n")

		print("Wait 2secs before greeting you, dear public!\n")

		showTime()
		let result = await sayAfter(secs: 2, who: "You")
		print("result: \(result)")
		showTime()

		print("\nWouha, 'tired. Let's sleep for 3secs!\n")

		showTime()
		let _ = await sleep(secs: 3)
		showTime()

		print("\nIs it really blocking? Nah. Let's greet Alice and Bob after resp. 2secs and 3secs _concurrently_!\n")

		async let alice = sayAfter(secs: 2, who: "Alice")
		async let bob = sayAfter(secs: 3, who: "Bob")

		showTime()
		let (result_alice, result_bob) = await (alice, bob)
		print("result_alice: \(result_alice)")
		print("result_bob: \(result_bob)")
		showTime()

		print("\nSee, it tooks 3secs, not 5secs!")
	}
}
