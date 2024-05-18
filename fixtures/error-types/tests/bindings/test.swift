import error_types
do {
    try oops()
    fatalError("Should have thrown")
} catch let e as ErrorInterface {
    let msg = "because uniffi told me so\n\nCaused by:\n    oops"
    assert(String(describing: e) == msg)
    assert(String(reflecting: e) == "ErrorInterface { e: \(msg) }")
}

do {
    try oops()
    fatalError("Should have thrown")
} catch {
    let msg = "because uniffi told me so\n\nCaused by:\n    oops"
    assert(String(describing: error) == msg)
    assert(String(reflecting: error) == "ErrorInterface { e: \(msg) }")
    assert(error.localizedDescription == "ErrorInterface { e: \(msg) }")
}

do {
    try oopsEnum()
    fatalError("Should have thrown")
} catch let e as Error {
    assert(e == Error.Oops)
    assert(String(describing: e) == "Oops")
    assert(String(reflecting: e) == "error_types.Error.Oops")
}
do {
    try oopsEnum()
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "Oops")
    assert(String(reflecting: error) == "error_types.Error.Oops")
    assert(error.localizedDescription == "error_types.Error.Oops")
}

do {
    try oopsNowrap()
    fatalError("Should have thrown")
} catch let e as ErrorInterface {
    assert(String(describing: e) == "because uniffi told me so\n\nCaused by:\n    oops")
}

do {
    try toops()
    fatalError("Should have thrown")
} catch let e as ErrorTrait {
    assert(e.msg() == "trait-oops")
}

let e = getError(message: "the error")
assert(String(describing: e) == "the error")
assert(String(reflecting: e) == "ErrorInterface { e: the error }")
// assert(Error.self is Swift.Error.Type) -- always true!
assert(Error.self != Swift.Error.self)
