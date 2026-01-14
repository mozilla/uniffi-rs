/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_bindgen_tests

var dispatchGroup = DispatchGroup()

// Primitive types
dispatchGroup.enter()
Task {
    let returnedU8 = await asyncRoundtripU8(v: 42)
    assert(returnedU8 == 42)

    let returnedI8 = await asyncRoundtripI8(v: -42)
    assert(returnedI8 == -42)

    let returnedU16 = await asyncRoundtripU16(v: 42)
    assert(returnedU16 == 42)

    let returnedI16 = await asyncRoundtripI16(v: -42)
    assert(returnedI16 == -42)

    let returnedU32 = await asyncRoundtripU32(v: 42)
    assert(returnedU32 == 42)

    let returnedI32 = await asyncRoundtripI32(v: -42)
    assert(returnedI32 == -42)

    let returnedU64 = await asyncRoundtripU64(v: 42)
    assert(returnedU64 == 42)

    let returnedI64 = await asyncRoundtripI64(v: -42)
    assert(returnedI64 == -42)

    let returnedF32 = await asyncRoundtripF32(v: 0.5)
    assert(returnedF32 == 0.5)

    let returnedF64 = await asyncRoundtripF64(v: -0.5)
    assert(returnedF64 == -0.5)

    let returnedString = await asyncRoundtripString(v: "hi")
    assert(returnedString == "hi")

    let returnedList = await asyncRoundtripVec(v: [42])
    assert(returnedList == [42])

    let returnedMap = await asyncRoundtripMap(v: ["hello": "world"])
    assert(returnedMap == ["hello": "world"])

    dispatchGroup.leave()
}
dispatchGroup.wait()

// Errors
dispatchGroup.enter()
Task {
    do {
        try await asyncThrowError()
        fatalError("expected asyncThrowError to throw")
    } catch TestError.Failure1 {
        // Expected
    } catch {
        fatalError("unexpected error \(error)")
    }
  dispatchGroup.leave()
}
dispatchGroup.wait()

// Interfaces/methods
dispatchGroup.enter()
Task {
    let obj = AsyncInterface(name: "Alice")
    let objName = await obj.name()
    assert(objName == "Alice")

    let obj2 = await asyncRoundtripObj(v: obj)
    let obj2Name = await obj2.name()
    assert(obj2Name == "Alice")

    dispatchGroup.leave()
}
dispatchGroup.wait()

// Callback interfaces
var asyncCallbackRefCount = 0
class AsyncCallbackImpl: TestAsyncCallbackInterface, @unchecked Sendable {
    var value: UInt32;

    init(value: UInt32) {
        self.value = value;
        asyncCallbackRefCount += 1
    }

    deinit {
        asyncCallbackRefCount -= 1
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

dispatchGroup.enter()
Task {
    let cbi = AsyncCallbackImpl(value: 42);
    await invokeTestAsyncCallbackInterfaceNoop(cbi: cbi);

    var value = await invokeTestAsyncCallbackInterfaceGetValue(cbi: cbi)
    assert(value == 42)

    await invokeTestAsyncCallbackInterfaceSetValue(cbi: cbi, value: 43);
    value = await invokeTestAsyncCallbackInterfaceGetValue(cbi: cbi)
    assert(value == 43)

    do {
        let _ = try await invokeTestAsyncCallbackInterfaceThrowIfEqual(
            cbi: cbi,
            numbers: CallbackInterfaceNumbers(a: 10, b: 10)
        )
    } catch TestError.Failure1 {
        // expected
    } catch {
        fatalError("unexpected error \(error)")
    } 

    let returnedNumbers = try! await invokeTestAsyncCallbackInterfaceThrowIfEqual(
        cbi: cbi,
        numbers: CallbackInterfaceNumbers(a: 10, b: 11)
    )
    assert(returnedNumbers == CallbackInterfaceNumbers(a: 10, b: 11))

    dispatchGroup.leave()
}
dispatchGroup.wait()

