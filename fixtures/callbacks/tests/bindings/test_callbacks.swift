/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if canImport(callbacks)
    import callbacks
#endif

// A bit more systematic in testing, but this time in English.
//
// 1. Pass in the callback as arguments.
// Make the callback methods use multiple aruments, with a variety of types, and
// with a variety of return types.
let rustGetters = RustGetters()
class KotlinGetters: ForeignGetters {
    func getBool(v: Bool, arg2: Bool) -> Bool { v != arg2 }
    func getString(v: String, arg2: Bool) -> String { arg2 ? "1234567890123" : v }
    func getOption(v: String?, arg2: Bool) -> String? { arg2 ? v?.uppercased() : v }
    func getList(v: [Int32], arg2: Bool) -> [Int32] { arg2 ? v : [] }
}

func test() {
    let callback = KotlinGetters()
    [true, false].forEach { v in
        let flag = true
        let expected = callback.getBool(v: v, arg2: flag)
        let observed = rustGetters.getBool(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: $expected != $observed")
    }

    [[Int32(1), Int32(2)], [Int32(0), Int32(1)]].forEach { v in
        let flag = true
        let expected = callback.getList(v: v, arg2: flag)
        let observed = rustGetters.getList(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: $expected != $observed")
    }

    ["Hello", "world"].forEach { v in
        let flag = true
        let expected = callback.getString(v: v, arg2: flag)
        let observed = rustGetters.getString(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: $expected != $observed")
    }

    ["Some", nil].forEach { v in
        let flag = false
        let expected = callback.getOption(v: v, arg2: flag)
        let observed = rustGetters.getOption(callback: callback, v: v, arg2: flag)
        assert(expected == observed, "roundtripping through callback: $expected != $observed")
    }

    assert(rustGetters.getStringOptionalCallback(callback: callback, v: "TestString", arg2: false) == "TestString")
    assert(rustGetters.getStringOptionalCallback(callback: nil, v: "TestString", arg2: false) == nil)

    // rustGetters.destroy()

    // 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
    // This is crucial if we want to configure a system at startup,
    // then use it without passing callbacks all the time.

    class StoredKotlinStringifier: StoredForeignStringifier {
        func fromSimpleType(value: Int32) -> String { "kotlin: \(value)" }
        // We don't test this, but we're checking that the arg type is included in the minimal list of types used
        // in the UDL.
        // If this doesn't compile, then look at TypeResolver.
        func fromComplexType(values: [Double?]?) -> String { "kotlin: \(values)" }
    }

    let kotlinStringifier = StoredKotlinStringifier()
    let rustStringifier = RustStringifier(callback: kotlinStringifier)
    ([1, 2] as [Int32]).forEach { v in
        let expected = kotlinStringifier.fromSimpleType(value: v)
        let observed = rustStringifier.fromSimpleType(value: v)
        assert(expected == observed, "callback is sent on construction: \(expected) != \(observed)")
    }

}
