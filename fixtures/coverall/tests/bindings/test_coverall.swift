/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import coverall

// TODO: use an actual test runner.


// Floats should be "close enough" for testing purposes.
fileprivate extension Double {
    func almostEquals(_ other: Double) -> Bool {
        return abs(self - other) < 0.000001
    }
}

fileprivate extension Float {
    func almostEquals(_ other: Float) -> Bool {
        return abs(self - other) < 0.000001
    }
}

// Test some_dict().
do {
    let d = createSomeDict()
    assert(d.text == "text")
    assert(d.maybeText == "maybe_text")
    assert(d.someBytes == Data("some_bytes".utf8))
    assert(d.maybeSomeBytes == Data("maybe_some_bytes".utf8))
    assert(d.aBool)
    assert(d.maybeABool == false);
    assert(d.unsigned8 == 1)
    assert(d.maybeUnsigned8 == 2)
    assert(d.unsigned16 == 3)
    assert(d.maybeUnsigned16 == 4)
    assert(d.unsigned64 == 18446744073709551615)
    assert(d.maybeUnsigned64 == 0)
    assert(d.signed8 == 8)
    assert(d.maybeSigned8 == 0)
    assert(d.signed64 == 9223372036854775807)
    assert(d.maybeSigned64 == 0)
    assert(d.float32.almostEquals(1.2345))
    assert(d.maybeFloat32!.almostEquals(22.0/7.0))
    assert(d.float64.almostEquals(0.0))
    assert(d.maybeFloat64!.almostEquals(1.0))
    assert(d.coveralls!.getName() == "some_dict")
}

// Test none_dict().
do {
    let d = createNoneDict()
    assert(d.text == "text")
    assert(d.maybeText == nil)
    assert(d.someBytes == Data("some_bytes".utf8))
    assert(d.maybeSomeBytes == nil)
    assert(d.aBool)
    assert(d.maybeABool == nil);
    assert(d.unsigned8 == 1)
    assert(d.maybeUnsigned8 == nil)
    assert(d.unsigned16 == 3)
    assert(d.maybeUnsigned16 == nil)
    assert(d.unsigned64 == 18446744073709551615)
    assert(d.maybeUnsigned64 == nil)
    assert(d.signed8 == 8)
    assert(d.maybeSigned8 == nil)
    assert(d.signed64 == 9223372036854775807)
    assert(d.maybeSigned64 == nil)
    assert(d.float32.almostEquals(1.2345))
    assert(d.maybeFloat32 == nil)
    assert(d.float64.almostEquals(0.0))
    assert(d.maybeFloat64 == nil)
    assert(d.coveralls == nil)
}

// Test arcs.
do {
    let coveralls = Coveralls(name: "test_arcs")
    assert(getNumAlive() == 1)
    // One ref held by the foreign-language code, one created for this method call.
    assert(coveralls.strongCount() == 2)
    assert(coveralls.getOther() == nil)
    coveralls.takeOther(other: coveralls)
    // Should now be a new strong ref, held by the object's reference to itself.
    assert(coveralls.strongCount() == 3)
    // But the same number of instances.
    assert(getNumAlive() == 1)
    // It's the same Rust object.
    assert(coveralls.getOther()!.getName() == "test_arcs")
    do {
        try coveralls.takeOtherFallible()
        fatalError("Should have thrown")
    } catch CoverallError.TooManyHoles {
        // It's okay!
    }
    // TODO: kinda hard to test this, as it triggers a fatal error.
    // coveralls!.takeOtherPanic(message: "expected panic: with an arc!")
    // do {
    //     try coveralls.falliblePanic(message: "Expected Panic!!")
    // } catch CoverallError.TooManyHoles {
    //     fatalError("Should have panicked!")
    // }
    coveralls.takeOther(other: nil);
    assert(coveralls.strongCount() == 2);
}

// Test simple errors
do {
    let coveralls = Coveralls(name: "test_simple_errors")

    assert(try! coveralls.maybeThrow(shouldThrow: false) == true)

    do {
        let _ = try coveralls.maybeThrow(shouldThrow: true)
        fatalError("Should have thrown")
    } catch CoverallError.TooManyHoles(let message) {
        // It's okay!
        assert(message == "The coverall has too many holes")
    }

    do {
        let _ = try coveralls.maybeThrowInto(shouldThrow: true)
        fatalError("Should have thrown")
    } catch CoverallError.TooManyHoles {
        // It's okay!
    }

    // Note: Can't test coveralls.panic() because rust panics trigger a fatal error in swift
}

// Test complex errors
do {
    let coveralls = Coveralls(name: "test_complex_errors")

    assert(try! coveralls.maybeThrowComplex(input: 0) == true)

    do {
        let _ = try coveralls.maybeThrowComplex(input: 1)
        fatalError("should have thrown")
    } catch let e as ComplexError {
        if case let .OsError(code, extendedCode) = e {
            assert(code == 10)
            assert(extendedCode == 20)
        } else {
            fatalError("wrong error variant: \(e)")
        }
        assert(String(describing: e) == "OsError(code: 10, extendedCode: 20)", "Unexpected ComplexError.OsError description: \(e)")
    }

    do {
        let _ = try coveralls.maybeThrowComplex(input: 2)
        fatalError("should have thrown")
    } catch let e as ComplexError {
        if case let .PermissionDenied(reason) = e {
            assert(reason == "Forbidden")
        } else {
            fatalError("wrong error variant: \(e)")
        }
        assert(String(describing: e) == "PermissionDenied(reason: \"Forbidden\")", "Unexpected ComplexError.PermissionDenied description: \(e)")
    }

    do {
        let _ = try coveralls.maybeThrowComplex(input: 3)
        fatalError("should have thrown")
    } catch let e as ComplexError {
        if case .UnknownError = e {
        } else {
            fatalError("wrong error variant: \(e)")
        }
        assert(String(describing: e) == "UnknownError", "Unexpected ComplexError.UnknownError description: \(e)")
    }

    do {
        let _ = try coveralls.maybeThrowComplex(input: 4)
        fatalError("should have thrown")
    } catch {
        assert(String(describing: error) == "rustPanic(\"Invalid input\")")
    }

}

// Test error values, including error enums with error variants.
do {
    do {
        let _ = try throwRootError()
        fatalError("should have thrown")
    } catch let e as RootError {
        if case let .Complex(error) = e {
            if case let .OsError(code, extendedCode) = error {
                assert(code == 1)
                assert(extendedCode == 2)
            } else {
                fatalError("wrong error variant: \(e)")
            }
        } else {
            fatalError("wrong error variant: \(e)")
        }
    }
    let e = getRootError();
    if case let .Other(error) = e {
        assert(error == OtherError.unexpected)
    } else {
        fatalError("wrong error variant: \(e)")
    }
    let e2 = getComplexError(e: nil);
    if case let .PermissionDenied(error) = e2 {
        assert(error == "too complex")
    } else {
        fatalError("wrong error variant: \(e)")
    }
    assert(getErrorDict(d: nil).complexError == nil)
}

// Swift GC is deterministic, `coveralls` is freed when it goes out of scope.
assert(getNumAlive() == 0);

// Test return objects
do {
    let coveralls = Coveralls(name: "test_return_objects")
    assert(getNumAlive() == 1)
    assert(coveralls.strongCount() == 2)
    do {
        let c2 = coveralls.cloneMe()
        assert(c2.getName() == coveralls.getName())
        assert(getNumAlive() == 2)
        assert(c2.strongCount() == 2)

        coveralls.takeOther(other: c2)
        // same number alive but `c2` has an additional ref count.
        assert(getNumAlive() == 2)
        assert(coveralls.strongCount() == 2)
        assert(c2.strongCount() == 3)
    }
    // We can drop Swifts's reference to `c2`, but the rust struct will not
    // be dropped as coveralls hold an `Arc<>` to it.
    assert(getNumAlive() == 2)
}

// Dropping `coveralls` will kill both.
assert(getNumAlive() == 0)

// Test a dict with defaults
// This does not call Rust code.
do {
    let d = DictWithDefaults()
    assert(d.name == "default-value")
    assert(d.category == nil)
    assert(d.integer == 31)

    let d2 = DictWithDefaults(name: "this", category: "that", integer: 42)
    assert(d2.name == "this")
    assert(d2.category == "that")
    assert(d2.integer == 42)
}

do {
    let coveralls = Coveralls(name: "test_dicts")

    let dict1 = coveralls.getDict(key: "answer", value: 42)
    assert(dict1["answer"] == 42)

    let dict2 = coveralls.getDict2(key: "answer", value: 42)
    assert(dict2["answer"] == 42)

    let dict3 = coveralls.getDict3(key: 31, value: 42)
    assert(dict3[31] == 42)
}

// Test interfaces as dict members
do {
    let coveralls = Coveralls(name: "test_interfaces_in_dicts")
    coveralls.addPatch(patch: Patch(color: Color.red))
    coveralls.addRepair(repair: Repair(when: Date.init(), patch: Patch(color: Color.blue)))
    assert(coveralls.getRepairs().count == 2)
}

// Test fallible constructors.
do {
    let _ = try FalliblePatch()
    fatalError("ctor should have thrown")
} catch {
    // OK!
}

do {
    let _ = try FalliblePatch.secondary()
    fatalError("ctor should have thrown")
} catch {
    // OK!
}

// Test bytes
do {
    let coveralls = Coveralls(name: "test_bytes")
    assert(coveralls.reverse(value: Data("123".utf8)) == Data("321".utf8))
}


struct SomeOtherError: Error { }


final class SwiftGetters: Getters {
    func getBool(v: Bool, arg2: Bool) -> Bool { v != arg2 }
    func getString(v: String, arg2: Bool) throws -> String {
        if v == "too-many-holes" {
            throw CoverallError.TooManyHoles(message: "Too many")
        }
        if v == "unexpected-error" {
            throw SomeOtherError()
        }
        return arg2 ? "HELLO" : v
    }
    func getOption(v: String, arg2: Bool) throws -> String? {
        if v == "os-error" {
            throw ComplexError.OsError(code: 100, extendedCode: 200)
        }
        if v == "unknown-error" {
            throw ComplexError.UnknownError
        }
        if arg2 {
            if !v.isEmpty {
                return v.uppercased()
            } else {
                return nil
            }
        } else {
            return v
        }
    }
    func getList(v: [Int32], arg2: Bool) -> [Int32] { arg2 ? v : [] }
    func getNothing(v: String) -> () {
    }

    func roundTripObject(coveralls: Coveralls) -> Coveralls {
        return coveralls
    }
}


// Test traits implemented in Rust
do {
    let getters = makeRustGetters()
    testGetters(g: getters)
    testGettersFromSwift(getters: getters)
}

// Test traits implemented in Swift
do {
    let getters = SwiftGetters()
    testGetters(g: getters)
    testGettersFromSwift(getters: getters)
}

func testGettersFromSwift(getters: Getters) {
    assert(getters.getBool(v: true, arg2: true) == false);
    assert(getters.getBool(v: true, arg2: false) == true);
    assert(getters.getBool(v: false, arg2: true) == true);
    assert(getters.getBool(v: false, arg2: false) == false);

    assert(try! getters.getString(v: "hello", arg2: false) == "hello");
    assert(try! getters.getString(v: "hello", arg2: true) == "HELLO");

    assert(try! getters.getOption(v: "hello", arg2: true) == "HELLO");
    assert(try! getters.getOption(v: "hello", arg2: false) == "hello");
    assert(try! getters.getOption(v: "", arg2: true) == nil);

    assert(getters.getList(v: [1, 2, 3], arg2: true) == [1, 2, 3])
    assert(getters.getList(v: [1, 2, 3], arg2: false) == [])

    assert(getters.getNothing(v: "hello") == ());

    do {
        let _ = try getters.getString(v: "too-many-holes", arg2: true)
        fatalError("should have thrown")
    } catch CoverallError.TooManyHoles {
        // Expected
    } catch {
        fatalError("Unexpected error: \(error)")
    }

    do {
        let _ = try getters.getOption(v: "os-error", arg2: true)
        fatalError("should have thrown")
    } catch ComplexError.OsError(let code, let extendedCode) {
        assert(code == 100)
        assert(extendedCode == 200)
    } catch {
        fatalError("Unexpected error: \(error)")
    }

    do {
        let _ = try getters.getOption(v: "unknown-error", arg2: true)
        fatalError("should have thrown")
    } catch ComplexError.UnknownError {
        // Expected
    } catch {
        fatalError("Unexpected error: \(error)")
    }

    do {
        let _ = try getters.getString(v: "unexpected-error", arg2: true)
    } catch {
        // Expected
    }
}

final class SwiftNode: NodeTrait, @unchecked Sendable {
    var p: NodeTrait? = nil

    func name() -> String {
        return "node-swift"
    }

    func setParent(parent: NodeTrait?) {
        self.p = parent
    }

    func getParent() -> NodeTrait? {
        return self.p
    }

    func strongCount() -> UInt64 {
        return 0 // TODO
    }
}

// Test Node trait
do {
    let traits = getTraits()
    assert(traits[0].name() == "node-1")
    // Note: strong counts are 1 more than you might expect, because the strongCount() method
    // holds a strong ref.
    assert(traits[0].strongCount() == 2)

    assert(traits[1].name() == "node-2")
    assert(traits[1].strongCount() == 2)

    // Note: this doesn't increase the Rust strong count, since we wrap the Rust impl with a
    // Swift impl before passing it to `set_parent()`
    traits[0].setParent(parent: traits[1])
    assert(ancestorNames(node: traits[0]) == ["node-2"])
    assert(ancestorNames(node: traits[1]) == [])
    assert(traits[1].strongCount() == 2)
    assert(traits[0].getParent()!.name() == "node-2")

    // Throw in a Swift implementation of the trait
    // The ancestry chain now goes traits[0] -> traits[1] -> swiftNode
    let swiftNode = SwiftNode()
    traits[1].setParent(parent: swiftNode)
    assert(ancestorNames(node: traits[0]) == ["node-2", "node-swift"])
    assert(ancestorNames(node: traits[1]) == ["node-swift"])
    assert(ancestorNames(node: swiftNode) == [])

    // Rotating things.
    // The ancestry chain now goes swiftNode -> traits[0] -> traits[1]
    traits[1].setParent(parent: nil)
    swiftNode.setParent(parent: traits[0])
    assert(ancestorNames(node: swiftNode) == ["node-1", "node-2"])
    assert(ancestorNames(node: traits[0]) == ["node-2"])
    assert(ancestorNames(node: traits[1]) == [])

    // Make sure we don't crash when undoing it all
    swiftNode.setParent(parent: nil)
    traits[0].setParent(parent: nil)
}

// A struct which implements the node trait.
do {
    let n = Node(name: "node")
    assert(String(describing: n).starts(with: "Node { name: Some(\"node\"), parent: Mutex { "))
    assert(n.getParent()?.name() == "via node")

    n.setParent(parent: n.getParent())
    // doubly-wrapped :(
    // Get: "Some(UniFFICallbackHandlerNodeTrait { handle: 19 })"
    // Want: Like the Rust node above.
    // debugPrint("parent \(n.describeParent())")

    let rustParent = Node(name: "parent")
    n.setParent(parent: rustParent)
    assert(n.getParent()?.name() == "parent")

    let swiftParent = SwiftNode()
    rustParent.setParent(parent: swiftParent)
    assert(ancestorNames(node: n) == ["parent", "node-swift"])
}

// Test round tripping
do {
    let rustGetters = makeRustGetters()
    // Check that these don't cause use-after-free bugs
    let _ = testRoundTripThroughRust(getters: rustGetters)

    testRoundTripThroughForeign(getters: SwiftGetters())
}

// Test rust-only traits
do {
    let stringUtils = getStringUtilTraits()
    assert(stringUtils[0].concat(a: "cow", b: "boy") == "cowboy")
    assert(stringUtils[1].concat(a: "cow", b: "boy") == "cowboy")
}

// Test HTMLError
do {
    try validateHtml(source: "test")
    fatalError("should have thrown")
} catch HtmlError.InvalidHtml {
    // Expected
} catch {
    fatalError("Unexpected error: \(error)")
}
