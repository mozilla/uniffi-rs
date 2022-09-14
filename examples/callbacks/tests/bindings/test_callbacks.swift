/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if canImport(callbacks)
    import callbacks
#endif

struct SomeOtherError: Error { }

// Simple example just to see it work.
// Pass in a string, get a string back.
// Pass in nothing, get unit back.
class OnCallAnsweredImpl : CallAnswerer {
    var mode: String

    init(withMode: String) {
        mode = withMode
    }

    func answer() throws -> String {
        switch mode {
            case "ready":
                return "Bonjour"

            case "busy":
                throw TelephoneError.Busy(message: "I'm Busy")

            default:
                throw SomeOtherError()
        }
    }
}

let telephone = Telephone()

assert(try! telephone.call(answerer: OnCallAnsweredImpl(withMode: "ready")) == "Bonjour")

do {
    _ = try telephone.call(answerer: OnCallAnsweredImpl(withMode: "busy"))
} catch TelephoneError.Busy {
    // Expected error
}

do {
    _ = try telephone.call(answerer: OnCallAnsweredImpl(withMode: "unexpected-value"))
} catch TelephoneError.InternalTelephoneError {
    // Expected error
}
