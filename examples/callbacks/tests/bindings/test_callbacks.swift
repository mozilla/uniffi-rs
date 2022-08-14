/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if canImport(callbacks)
    import callbacks
#endif

// Simple example just to see it work.
// Pass in a string, get a string back.
// Pass in nothing, get unit back.
class OnCallAnsweredImpl : OnCallAnswered {
    var yesCount: Int = 0
    var busyCount: Int = 0
    var stringReceived = ""

    func hello() -> String {
        yesCount += 1
        return "Hi hi \(yesCount)"
    }

    func busy() {
        busyCount += 1
    }

    func textReceived(text: String) {
        stringReceived = text
    }
}

let sim = getSimCards()[0]
let cbObject = OnCallAnsweredImpl()
let telephone = Telephone()

telephone.call(sim: sim, domestic: true, callResponder: cbObject)
assert(cbObject.busyCount == 0, "yesCount=\(cbObject.busyCount) (should be 0)")
assert(cbObject.yesCount == 1, "yesCount=\(cbObject.yesCount) (should be 1)")

telephone.call(sim: sim, domestic: true, callResponder: cbObject)
assert(cbObject.busyCount == 0, "yesCount=\(cbObject.busyCount) (should be 0)")
assert(cbObject.yesCount == 2, "yesCount=\(cbObject.yesCount) (should be 2)")

telephone.call(sim: sim, domestic: false, callResponder: cbObject)
assert(cbObject.busyCount == 1, "yesCount=\(cbObject.busyCount) (should be 1)")
assert(cbObject.yesCount == 2, "yesCount=\(cbObject.yesCount) (should be 2)")

let cbObject2 = OnCallAnsweredImpl()
telephone.call(sim: sim, domestic: true, callResponder: cbObject2)
assert(cbObject2.busyCount == 0, "yesCount=\(cbObject2.busyCount) (should be 0)")
assert(cbObject2.yesCount == 1, "yesCount=\(cbObject2.yesCount) (should be 1)")

