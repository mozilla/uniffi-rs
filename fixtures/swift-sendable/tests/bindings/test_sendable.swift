// swift-version: 6

import swift_sendable

// This test verifies all the types listed below in `SendablesOnly` are `Sendable`.
// There is no assertions, because compilation will fail if any of the types are not `Sendable`.

struct SendablesOnly: Sendable {
    var record: UniffiRecord
    var object: UniffiObject
    var nested: Nested
}
