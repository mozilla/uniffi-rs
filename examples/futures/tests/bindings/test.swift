import uniffi_example_futures
import Foundation

var counter = DispatchGroup()

counter.enter()

Task {
    let result = await sayAfter(ms: 20, who: "Alice")
    assert(result == "Hello, Alice!")

    let store = Store(backgroundExecutor: UniFfiForeignExecutor(priority: TaskPriority.background))
    let result2 = await store.loadItem()
    assert(result2 == "this was loaded from disk")
    counter.leave()
}

counter.wait()
