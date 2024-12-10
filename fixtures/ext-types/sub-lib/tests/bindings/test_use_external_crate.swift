import imported_types_sublib
import Foundation

var counter = DispatchGroup()

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
