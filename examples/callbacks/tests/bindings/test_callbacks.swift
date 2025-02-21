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
final class OnCallAnsweredImpl : CallAnswerer {
    let mode: String

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
let sim = getSimCards()[0];

assert(try! telephone.call(sim: sim, answerer: OnCallAnsweredImpl(withMode: "ready")) == "Bonjour")

// We can implement our own sim cards.
final class Sim : SimCard {
    func name() -> String {
        return "swift"
    }
}

assert(try! telephone.call(sim: Sim(), answerer: OnCallAnsweredImpl(withMode: "ready")) == "swift est bon marché")

// Error cases.
do {
    _ = try telephone.call(sim: sim, answerer: OnCallAnsweredImpl(withMode: "busy"))
} catch TelephoneError.Busy {
    // Expected error
}

do {
    _ = try telephone.call(sim: sim, answerer: OnCallAnsweredImpl(withMode: "unexpected-value"))
} catch TelephoneError.InternalTelephoneError {
    // Expected error
}
