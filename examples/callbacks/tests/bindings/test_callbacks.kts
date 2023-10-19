/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.callbacks.*

class SomeOtherError: Exception()

// Simple example just to see it work.
// Pass in a string, get a string back.
// Pass in nothing, get unit back.
class CallAnswererImpl(val mode: String) : CallAnswerer {
    override fun answer(): String {
        if (mode == "normal") {
            return "Bonjour"
        } else if (mode == "busy") {
            throw TelephoneException.Busy("I'm busy")
        } else {
            throw SomeOtherError();
        }
    }
}

val telephone = Telephone()
val sim = getSimCards()[0]

assert(telephone.call(sim, CallAnswererImpl("normal")) == "Bonjour")

// Our own sim.
class Sim() : SimCard {
    override fun name(): String {
        return "kotlin"
    }
}
assert(telephone.call(Sim(), CallAnswererImpl("normal")) == "kotlin est bon marché")

try {
    telephone.call(sim, CallAnswererImpl("busy"))
    throw RuntimeException("Should have thrown a Busy exception!")
} catch(e: TelephoneException.Busy) {
    // It's okay
}

try {
    telephone.call(sim, CallAnswererImpl("something-else"))
    throw RuntimeException("Should have thrown an internal exception!")
} catch(e: TelephoneException.InternalTelephoneException) {
    // It's okay
}

telephone.destroy()
