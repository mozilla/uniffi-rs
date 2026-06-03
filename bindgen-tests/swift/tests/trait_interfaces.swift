/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_bindgen_tests

var dispatchGroup = DispatchGroup()
var traitRefCount = 0;

class TraitImpl: TestTraitInterface, @unchecked Sendable {
    var value: UInt32;

    init(value: UInt32) {
        self.value = value;
        traitRefCount += 1
    }

    deinit {
        traitRefCount -= 1
    }

    func noop() { }

    func getValue() -> UInt32 {
        return self.value;
    }

    func setValue(value: UInt32) {
        self.value = value;
    }

    func throwIfEqual(numbers: CallbackInterfaceNumbers) throws -> CallbackInterfaceNumbers {
        if numbers.a == numbers.b {
            throw TestError.Failure1
        }
        return numbers
    }

    func roundtripRecord(value: SimpleRec) -> SimpleRec { value }

    func roundtripEnum(value: EnumWithData) -> EnumWithData { value }

    func roundtripInterface(value: TestInterface) -> TestInterface { value }
}

// Test calling the Rust impl from Swift
func testRustImpl(traitImpl: TestTraitInterface) {
    traitImpl.noop();
    assert(traitImpl.getValue() == 42)
    traitImpl.setValue(value: 43);
    assert(traitImpl.getValue() == 43)
    do {
        let _ = try traitImpl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 10))
        fatalError("expected throwIfEqual to throw")
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    }

    assert(
        try! traitImpl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 11)) ==
        CallbackInterfaceNumbers(a: 10, b: 11))

    assert(traitImpl.roundtripRecord(value: SimpleRec(a: 10)) == SimpleRec(a: 10))
    assert(
        traitImpl.roundtripEnum(value: EnumWithData.a(value: 10, value2: 20)) ==
        EnumWithData.a(value: 10, value2: 20))
    assert(traitImpl.roundtripInterface(value: TestInterface(value: 20)).getValue() == 20)
}

testRustImpl(traitImpl: createTestTraitInterface(value: 42))
testRustImpl(traitImpl: roundtripTestTraitInterface(interface: createTestTraitInterface(value: 42)))
testRustImpl(traitImpl: roundtripTestTraitInterfaceList(interfaces: [createTestTraitInterface(value: 42)])[0])

// Test calling the Swift impl by roundtripping it through a Rust function
func testSwiftImpl(traitImpl: TestTraitInterface) {
    invokeTestTraitInterfaceNoop(interface: traitImpl)
    assert(invokeTestTraitInterfaceGetValue(interface: traitImpl) == 42)
    invokeTestTraitInterfaceSetValue(interface: traitImpl, value: 43)
    assert(invokeTestTraitInterfaceGetValue(interface: traitImpl) == 43)
    do {
        let _ = try invokeTestTraitInterfaceThrowIfEqual(
            interface: traitImpl,
            numbers: CallbackInterfaceNumbers(a: 10, b: 10)
        )
        fatalError("expected throwIfEqual to throw")
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    }

    assert(
        try! invokeTestTraitInterfaceThrowIfEqual(
            interface: traitImpl,
            numbers: CallbackInterfaceNumbers(a: 10, b: 11)
        ) == CallbackInterfaceNumbers(a: 10, b: 11))

    assert(
        invokeTestTraitInterfaceRoundtripRecord(interface: traitImpl, rec: SimpleRec(a: 10)) ==
        SimpleRec(a: 10))
    assert(
        invokeTestTraitInterfaceRoundtripEnum(interface: traitImpl, en: EnumWithData.a(value: 10, value2: 20)) ==
        EnumWithData.a(value: 10, value2: 20))
    assert(invokeTestTraitInterfaceRoundtripInterface(interface: traitImpl, iface: TestInterface(value: 20)).getValue() == 20)
}

testSwiftImpl(traitImpl: TraitImpl(value: 42))
testSwiftImpl(traitImpl: roundtripTestTraitInterface(interface: TraitImpl(value: 42)))
testSwiftImpl(traitImpl: roundtripTestTraitInterfaceList(interfaces: [TraitImpl(value: 42)])[0])

var asyncTraitRefCount = 0;

class AsyncTraitImpl: AsyncTestTraitInterface, @unchecked Sendable {
    var value: UInt32;

    init(value: UInt32) {
        self.value = value;
        asyncTraitRefCount += 1
    }

    deinit {
        asyncTraitRefCount -= 1
    }

    func noop() async { }

    func getValue() async -> UInt32 {
        return self.value;
    }

    func setValue(value: UInt32) async {
        self.value = value;
    }

    func throwIfEqual(numbers: CallbackInterfaceNumbers) async throws -> CallbackInterfaceNumbers {
        if numbers.a == numbers.b {
            throw TestError.Failure1
        }
        return numbers
    }
}

func testAsyncRustImpl(traitImpl: AsyncTestTraitInterface) async {
    await traitImpl.noop();
    var valueReturn = await traitImpl.getValue();
    assert(valueReturn == 42)
        await traitImpl.setValue(value: 43);
    valueReturn = await traitImpl.getValue();
    assert(valueReturn == 43)
        do {
            let _ = try await traitImpl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 10))
                fatalError("expected throwIfEqual to throw")
        } catch TestError.Failure1 {
            // expected
        } catch {
            fatalError("unexpected error \(error)")
        }

    let numbersReturn = try! await traitImpl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 11))
    assert(numbersReturn == CallbackInterfaceNumbers(a: 10, b: 11))
}

dispatchGroup.enter()
Task {
    await testAsyncRustImpl(traitImpl: createAsyncTestTraitInterface(value: 42))
    await testAsyncRustImpl(traitImpl: roundtripAsyncTestTraitInterface(interface: createAsyncTestTraitInterface(value: 42)))
    await testAsyncRustImpl(traitImpl: roundtripAsyncTestTraitInterfaceList(interfaces: [createAsyncTestTraitInterface(value: 42)])[0])
    dispatchGroup.leave()
}
dispatchGroup.wait()

func testAsyncSwiftImpl(traitImpl: AsyncTestTraitInterface) async {
    await invokeAsyncTestTraitInterfaceNoop(interface: traitImpl)
    var val = await invokeAsyncTestTraitInterfaceGetValue(interface: traitImpl)
    assert(val == 42)
    await invokeAsyncTestTraitInterfaceSetValue(interface: traitImpl, value: 43)
    val = await invokeAsyncTestTraitInterfaceGetValue(interface: traitImpl)
    assert(val == 43)
    do {
        let _ = try await invokeAsyncTestTraitInterfaceThrowIfEqual(
            interface: traitImpl,
            numbers: CallbackInterfaceNumbers(a: 10, b: 10)
        )
        fatalError("expected throwIfEqual to throw")
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    }

    let numbersReturn = try! await invokeAsyncTestTraitInterfaceThrowIfEqual(
        interface: traitImpl,
        numbers: CallbackInterfaceNumbers(a: 10, b: 11)
    )
    assert(numbersReturn == CallbackInterfaceNumbers(a: 10, b: 11))
}

dispatchGroup.enter()
Task {
    await testAsyncSwiftImpl(traitImpl: AsyncTraitImpl(value: 42))
    await testAsyncSwiftImpl(traitImpl: roundtripAsyncTestTraitInterface(interface: AsyncTraitImpl(value: 42)))
    await testAsyncSwiftImpl(traitImpl: roundtripAsyncTestTraitInterfaceList(interfaces: [AsyncTraitImpl(value: 42)])[0])
    dispatchGroup.leave()
}
dispatchGroup.wait()

// The previous calls created and destroyed a ton of references to the Swift-implemented trait
// interfaces, check that the refcounts have gone back to 0.
assert(traitRefCount == 0)
assert(asyncTraitRefCount == 0)

// Rust-only trait: smoke-test for the #[uniffi::export(rust)] codegen path
func testRustOnlyImpl(impl: TestRustOnlyTraitInterface) {
    do {
        let _ = try impl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 10))
        fatalError("expected throwIfEqual to throw")
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    }
    assert(
        try! impl.throwIfEqual(numbers: CallbackInterfaceNumbers(a: 10, b: 11)) == CallbackInterfaceNumbers(a: 10, b: 11))
}

testRustOnlyImpl(impl: createRustOnlyTestTraitInterface())

// Foreign-only trait: smoke-test for the #[uniffi::export(foreign)] codegen path
class ForeignOnlyImpl: TestForeignOnlyTraitInterface, @unchecked Sendable {
    func throwIfEqual(numbers: CallbackInterfaceNumbers) throws -> CallbackInterfaceNumbers {
        if numbers.a == numbers.b {
            throw TestError.Failure1
        }
        return numbers
    }
}

func testForeignOnlyImpl(impl: TestForeignOnlyTraitInterface) {
    do {
        let _ = try invokeTestForeignOnlyTraitThrowIfEqual(
            interface: impl,
            numbers: CallbackInterfaceNumbers(a: 10, b: 10)
        )
        fatalError("expected throwIfEqual to throw")
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    }
    assert(
        try! invokeTestForeignOnlyTraitThrowIfEqual(
            interface: impl,
            numbers: CallbackInterfaceNumbers(a: 10, b: 11)
        ) == CallbackInterfaceNumbers(a: 10, b: 11))
}

testForeignOnlyImpl(impl: ForeignOnlyImpl())
testForeignOnlyImpl(impl: roundtripTestForeignOnlyTrait(interface: ForeignOnlyImpl()))
