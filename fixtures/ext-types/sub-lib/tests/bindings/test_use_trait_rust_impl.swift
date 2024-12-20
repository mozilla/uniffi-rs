import imported_types_sublib
import Foundation

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
    let impl = getTraitImpl()
    let rust = UniffiOneTraitWrapper(inner: impl)
    let result = rust.hello()
    assert(result == "sub-lib trait impl says hello")
    counter.leave()
}

counter.enter()
Task {
    let impl = getAsyncTraitImpl()
    let rust = UniffiOneAsyncTraitWrapper(inner: impl)
    let result = await rust.helloAsync()
    assert(result == "sub-lib async trait impl says hello")
    counter.leave()
}

counter.wait()
