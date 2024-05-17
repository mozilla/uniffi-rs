import error_types
do {
    try oops()
    fatalError("Should have thrown")
} catch let e as ErrorInterface {
    assert(String(describing: e) == "because uniffi told me so\n\nCaused by:\n    oops")
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

do {
    try oopsEnum()
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "Oops")
}

do {
    try oopsFlatInner()
    fatalError("Should have thrown")
} catch let e as Error {
    print(String(describing: e))
    assert(String(describing: e) == "??")
}

let e = getError(message: "the error")
assert(String(describing: e) == "the error")
assert(String(reflecting: e) == "ErrorInterface { e: the error }")
assert(Error.self is Swift.Error.Type)
assert(Error.self != Swift.Error.self)
