/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    //     fatalError("Should have paniced!")
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
    } catch CoverallError.TooManyHoles {
        // It's okay!
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
    } catch ComplexError.OsError(let code, let extendedCode) {
        assert(code == 10)
        assert(extendedCode == 20)
    }

    do {
        let _ = try coveralls.maybeThrowComplex(input: 2)
        fatalError("should have thrown")
    } catch ComplexError.PermissionDenied(let reason) {
        assert(reason == "Forbidden")
    }

    do {
        let _ = try coveralls.maybeThrowComplex(input: 3)
        fatalError("should have thrown")
    } catch {
        assert(String(describing: error) == "rustPanic(\"Invalid input\")")
    }

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
