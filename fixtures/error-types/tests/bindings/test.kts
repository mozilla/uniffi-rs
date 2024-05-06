/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.error_types.*;
import kotlinx.coroutines.*

try {
    oops()
    throw RuntimeException("Should have failed")
} catch (e: ErrorInterface) {
    assert(e.toString() == "because uniffi told me so\n\nCaused by:\n    oops")
    assert(e.chain().size == 2)
    assert(e.link(0U) == "because uniffi told me so")
}

try {
    oopsNowrap()
    throw RuntimeException("Should have failed")
} catch (e: ErrorInterface) {
    assert(e.toString() == "because uniffi told me so\n\nCaused by:\n    oops")
    assert(e.chain().size == 2)
    assert(e.link(0U) == "because uniffi told me so")
}

try {
    toops()
    throw RuntimeException("Should have failed")
} catch (e: ErrorTrait) {
    assert(e.msg() == "trait-oops")
}

val e = getError("the error")
assert(e.toString() == "the error")
assert(e.link(0U) == "the error")

try {
    throwRich("oh no")
    throw RuntimeException("Should have failed")
} catch (e: RichException) {
    assert(e.toString() == "RichError: \"oh no\"")
}

runBlocking {
    try {
        aoops()
        throw RuntimeException("Should have failed")
    } catch (e: ErrorInterface) {
        assert(e.toString() == "async-oops")
    }
}
