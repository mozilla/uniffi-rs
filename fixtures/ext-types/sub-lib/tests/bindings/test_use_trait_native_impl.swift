import imported_types_sublib
import Foundation

final class SwiftImpl: UniffiOneAsyncTrait, UniffiOneTrait {
    func hello() -> String {
        "Hello from Swift"
    }

    func helloAsync() async -> String {
        "Hello async from Swift"
    }
}

var counter = DispatchGroup()

// Add 'ISSUE_2343' env var to `cargo test` command to reproduce the crash.
// https://github.com/mozilla/uniffi-rs/issues/2343
if ProcessInfo.processInfo.environment["ISSUE_2343"] == nil {
    counter.enter()
    Task {
        let impl = getTraitImpl()
        let result = impl.hello()
        assert(result == "sub-lib trait impl says hello")
        counter.leave()
    }

    counter.enter()
    Task {
        let impl = getAsyncTraitImpl()
        let result = await impl.helloAsync()
        assert(result == "sub-lib async trait impl says hello")
        counter.leave()
    }

    counter.wait()
}

counter.enter()
Task {
    let impl = SwiftImpl()
    let rust = UniffiOneTraitWrapper(inner: impl)
    let result = rust.hello()
    assert(result == "Hello from Swift")
    counter.leave()
}

counter.enter()
Task {
    let impl = SwiftImpl()
    let rust = UniffiOneAsyncTraitWrapper(inner: impl)
    let result = await rust.helloAsync()
    assert(result == "Hello async from Swift")
    counter.leave()
}

counter.wait()
