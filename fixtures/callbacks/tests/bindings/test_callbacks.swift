/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if canImport(fixture_callbacks)
    import fixture_callbacks
#endif

struct SomeOtherError: Error { }

// A bit more systematic in testing, but this time in English.
//
// 1. Pass in the callback as arguments.
// Make the callback methods use multiple aruments, with a variety of types, and
// with a variety of return types.
let rustGetters = RustGetters()
class SwiftGetters: ForeignGetters {
    func getBool(v: Bool, argumentTwo: Bool) throws -> Bool { v != argumentTwo }
    func getString(v: String, arg2: Bool) throws -> String { 
        if v == "bad-argument" {
            throw SimpleError.BadArgument(message: "Bad argument")
        }
        if v == "unexpected-error" {
            throw SomeOtherError()
        }
        return arg2 ? "1234567890123" : v
    }
    func getOption(v: String?, arg2: Bool) throws -> String? {
        if v == "bad-argument" {
            throw ComplexError.ReallyBadArgument(code: 20)
        }
        if v == "unexpected-error" {
            throw SomeOtherError()
        }
        return arg2 ? v?.uppercased() : v
    }
    func getList(v: [Int32], arg2: Bool) throws -> [Int32] { arg2 ? v : [] }
}

do {
    let callback = SwiftGetters()
    [true, false].forEach { v in
        let flag = true
        let expected = try! callback.getBool(v: v, argumentTwo: flag)
        let observed = try! rustGetters.getBool(callback: callback, v: v, argumentTwo: flag)
        assert(expected == observed, "roundtripping through callback: \(String(describing: expected)) != \(String(describing: observed))")
    }

    [[Int32(1), Int32(2)], [Int32(0), Int32(1)]].forEach { v in
        let flag = true
        let expected = try! callback.getList(v: v, arg2: flag)
        let observed = try! rustGetters.getList(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: \(String(describing: expected)) != \(String(describing: observed))")
    }

    ["Hello", "world"].forEach { v in
        let flag = true
        let expected = try! callback.getString(v: v, arg2: flag)
        let observed = try! rustGetters.getString(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: \(String(describing: expected)) != \(String(describing: observed))")
    }

    ["Some", nil].forEach { v in
        let flag = false
        let expected = try! callback.getOption(v: v, arg2: flag)
        let observed = try! rustGetters.getOption(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: \(String(describing: expected)) != \(String(describing: observed))")
    }

    assert(try! rustGetters.getStringOptionalCallback(callback: callback, v: "TestString", arg2: false) == "TestString")
    assert(try! rustGetters.getStringOptionalCallback(callback: nil, v: "TestString", arg2: false) == nil)

    // rustGetters.destroy()

    // 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
    // This is crucial if we want to configure a system at startup,
    // then use it without passing callbacks all the time.

    class StoredSwiftStringifier: StoredForeignStringifier {
        func fromSimpleType(value: Int32) -> String { "swift: \(value)" }
        // We don't test this, but we're checking that the arg type is included in the minimal list of types used
        // in the UDL.
        // If this doesn't compile, then look at TypeResolver.
        func fromComplexType(values: [Double?]?) -> String { "swift: \(String(describing: values))" }
    }

    let swiftStringifier = StoredSwiftStringifier()
    let rustStringifier = RustStringifier(callback: swiftStringifier)
    ([1, 2] as [Int32]).forEach { v in
        let expected = swiftStringifier.fromSimpleType(value: v)
        let observed = rustStringifier.fromSimpleType(value: v)
        assert(expected == observed, "callback is sent on construction: \(expected) != \(observed)")
    }


    // 3. Error handling
    do {
        _ = try rustGetters.getString(callback: callback, v: "bad-argument", arg2: true)
        assertionFailure("getString() should have thrown an exception")
    } catch SimpleError.BadArgument {
        // Expected exception
    }

    do {
        _ = try rustGetters.getString(callback: callback, v: "unexpected-error", arg2: true)
        assertionFailure("getString() should have thrown an exception")
    } catch SimpleError.UnexpectedError {
        // Expected exception
    }

    do {
        _ = try rustGetters.getOption(callback: callback, v: "bad-argument", arg2: true)
        assertionFailure("getString() should have thrown an exception")
    } catch ComplexError.ReallyBadArgument(let code) {
        // Expected exception
        assert(code == 20)
    }

    do {
        _ = try rustGetters.getOption(callback: callback, v: "unexpected-error", arg2: true)
        assertionFailure("getString() should have thrown an exception")
    } catch ComplexError.UnexpectedErrorWithReason(let reason) {
        // Expected exception
        assert(reason == String(describing: SomeOtherError()))
    }
}
