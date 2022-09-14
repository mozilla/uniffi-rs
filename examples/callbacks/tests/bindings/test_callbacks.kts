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

assert(telephone.call(CallAnswererImpl("normal")) == "Bonjour")

try {
    telephone.call(CallAnswererImpl("busy"))
    throw RuntimeException("Should have thrown a Busy exception!")
} catch(e: TelephoneException.Busy) {
    // It's okay
}

try {
    telephone.call(CallAnswererImpl("something-else"))
    throw RuntimeException("Should have thrown a Busy exception!")
} catch(e: TelephoneException.InternalTelephoneException) {
    // It's okay
}

telephone.destroy()
