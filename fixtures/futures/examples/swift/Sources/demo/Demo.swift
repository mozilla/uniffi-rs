import uniffi_futures

@main
struct Testing {
	static func main() async throws {
		print("a")

		let x1 = await alwaysReady()
		print("alwaysReady: \(x1)")
		
		let x2 = await say()
		print("say: \(x2)")

		print("b")
	}
}
