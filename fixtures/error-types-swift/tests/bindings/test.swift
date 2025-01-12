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
    try oopsEnum(i: 0)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(e == Error.oops)
    assert(String(describing: e) == "oops")
    assert(String(reflecting: e) == "error_types.Error.oops")
}
do {
    try oopsEnum(i: 0)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "oops")
    assert(String(reflecting: error) == "error_types.Error.oops")
    assert(error.localizedDescription == "error_types.Error.oops")
}

do {
    try oopsEnum(i: 1)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "value(value: \"value\")")
    assert(String(reflecting: error) == "error_types.Error.value(value: \"value\")")
    assert(error.localizedDescription == "error_types.Error.value(value: \"value\")")
}

do {
    try oopsEnum(i: 2)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "intValue(value: 2)")
    assert(String(reflecting: error) == "error_types.Error.intValue(value: 2)")
    assert(error.localizedDescription == "error_types.Error.intValue(value: 2)")
}

do {
    try oopsEnum(i: 3)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "flatInnerError(error: error_types.FlatInner.caseA(message: \"inner\"))")
    assert(String(reflecting: e) == "error_types.Error.flatInnerError(error: error_types.FlatInner.caseA(message: \"inner\"))")
}

do {
    try oopsEnum(i: 4)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "flatInnerError(error: error_types.FlatInner.caseB(message: \"NonUniffiTypeValue: value\"))")
    assert(String(reflecting: e) == "error_types.Error.flatInnerError(error: error_types.FlatInner.caseB(message: \"NonUniffiTypeValue: value\"))")
}

do {
    try oopsEnum(i: 5)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "innerError(error: error_types.Inner.caseA(\"inner\"))")
}

do {
    try oopsTuple(i: 0)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "oops(\"oops\")")
    assert(String(reflecting: error) == "error_types.TupleError.oops(\"oops\")")
    assert(error.localizedDescription == "error_types.TupleError.oops(\"oops\")")
}

do {
    try oopsTuple(i: 1)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "value(1)")
    assert(String(reflecting: error) == "error_types.TupleError.value(1)")
    assert(error.localizedDescription == "error_types.TupleError.value(1)")
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
