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
    assert(e == Error.Oops)
    assert(String(describing: e) == "Oops")
    assert(String(reflecting: e) == "error_types.Error.Oops")
}
do {
    try oopsEnum(i: 0)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "Oops")
    assert(String(reflecting: error) == "error_types.Error.Oops")
    assert(error.localizedDescription == "error_types.Error.Oops")
}

do {
    try oopsEnum(i: 1)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "Value(value: \"value\")")
    assert(String(reflecting: error) == "error_types.Error.Value(value: \"value\")")
    assert(error.localizedDescription == "error_types.Error.Value(value: \"value\")")
}

do {
    try oopsEnum(i: 2)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "IntValue(value: 2)")
    assert(String(reflecting: error) == "error_types.Error.IntValue(value: 2)")
    assert(error.localizedDescription == "error_types.Error.IntValue(value: 2)")
}

do {
    try oopsEnum(i: 3)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "FlatInnerError(error: error_types.FlatInner.CaseA(message: \"inner\"))")
    assert(String(reflecting: e) == "error_types.Error.FlatInnerError(error: error_types.FlatInner.CaseA(message: \"inner\"))")
}

do {
    try oopsEnum(i: 4)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "FlatInnerError(error: error_types.FlatInner.CaseB(message: \"NonUniffiTypeValue: value\"))")
    assert(String(reflecting: e) == "error_types.Error.FlatInnerError(error: error_types.FlatInner.CaseB(message: \"NonUniffiTypeValue: value\"))")
}

do {
    try oopsEnum(i: 5)
    fatalError("Should have thrown")
} catch let e as Error {
    assert(String(describing: e) == "InnerError(error: error_types.Inner.CaseA(\"inner\"))")
}

do {
    try oopsTuple(i: 0)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "Oops(\"oops\")")
    assert(String(reflecting: error) == "error_types.TupleError.Oops(\"oops\")")
    assert(error.localizedDescription == "error_types.TupleError.Oops(\"oops\")")
}

do {
    try oopsTuple(i: 1)
    fatalError("Should have thrown")
} catch {
    assert(String(describing: error) == "Value(1)")
    assert(String(reflecting: error) == "error_types.TupleError.Value(1)")
    assert(error.localizedDescription == "error_types.TupleError.Value(1)")
}

do {
    try oopsCustom(i: 1)
    fatalError("Should have thrown")
} catch {
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
