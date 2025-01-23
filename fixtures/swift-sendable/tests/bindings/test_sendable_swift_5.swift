// swift-version: 5

import swift_sendable

func isSendableHack<T: Sendable>(_ type: T.Type) -> Bool { true }
func isSendableHack<T>(_ type: T.Type) -> Bool { false }
func assert<T>(_ type: T.Type, isSendable expected: Bool) {
    Swift.assert(isSendableHack(T.self) == expected, "Expect \(type) to \(expected ? "be" : "not be") Sendable")
}

assert(UniffiRecord.self, isSendable: false)
assert(UniffiObject.self, isSendable: false)
