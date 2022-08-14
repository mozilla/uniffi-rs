/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.callbacks.*

// Simple example just to see it work.
// Pass in a string, get a string back.
// Pass in nothing, get unit back.
class OnCallAnsweredImpl : OnCallAnswered {
    var yesCount: Int = 0
    var busyCount: Int = 0
    var stringReceived = ""

    override fun hello(): String {
        yesCount ++
        return "Hi hi $yesCount"
    }

    override fun busy() {
        busyCount ++
    }

    override fun textReceived(text: String) {
        stringReceived = text
    }
}

val sim = getSimCards()[0]
val cbObject = OnCallAnsweredImpl()
val telephone = Telephone()

telephone.call(sim, true, cbObject)
assert(cbObject.busyCount == 0) { "yesCount=${cbObject.busyCount} (should be 0)" }
assert(cbObject.yesCount == 1) { "yesCount=${cbObject.yesCount} (should be 1)" }

telephone.call(sim, true, cbObject)
assert(cbObject.busyCount == 0) { "yesCount=${cbObject.busyCount} (should be 0)" }
assert(cbObject.yesCount == 2) { "yesCount=${cbObject.yesCount} (should be 2)" }

telephone.call(sim, false, cbObject)
assert(cbObject.busyCount == 1) { "yesCount=${cbObject.busyCount} (should be 1)" }
assert(cbObject.yesCount == 2) { "yesCount=${cbObject.yesCount} (should be 2)" }

val cbObjet2 = OnCallAnsweredImpl()
telephone.call(sim, true, cbObjet2)
assert(cbObjet2.busyCount == 0) { "yesCount=${cbObjet2.busyCount} (should be 0)" }
assert(cbObjet2.yesCount == 1) { "yesCount=${cbObjet2.yesCount} (should be 1)" }

telephone.destroy()
