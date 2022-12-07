import uniffi_futures

@main
struct Testing {
	static func main() async throws {
		print("a")

		let x1 = await alwaysReady()
		print("alwaysReady: \(x1)")

		let x2 = await say()
		print("say: \(x2)")

		let x3 = await sayAfter(secs: 3, who: "World")
		print("say_after: \(x3)")

		let x4 = await sleep(secs: 2)
		print("sleep: \(x4)")

		let m = newMegaphone()
		let x5 = await m.sayAfter(secs: 2, who: "Gordon")
		print("Megaphone::say_after \(x5)")

		print("b")
	}
}
