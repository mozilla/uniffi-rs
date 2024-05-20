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
    oops()
    throw RuntimeException("Should have failed")
} catch (e: kotlin.Exception) {
    assert(e.toString() == "because uniffi told me so\n\nCaused by:\n    oops")
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

try {
    oopsEnum(0u)
    throw RuntimeException("Should have failed")
} catch (e: Exception) {
    assert(e.toString() == "uniffi.error_types.Exception${'$'}Oops: ")
}

try {
    oopsEnum(1u)
    throw RuntimeException("Should have failed")
} catch (e: Exception) {
    assert(e.toString() == "uniffi.error_types.Exception${'$'}Value: value=value")
}

try {
    oopsEnum(2u)
    throw RuntimeException("Should have failed")
} catch (e: Exception) {
    assert(e.toString() == "uniffi.error_types.Exception${'$'}IntValue: value=2")
}

try {
    oopsTuple(0u)
    throw RuntimeException("Should have failed")
} catch (e: TupleException) {
    assert(e.toString() == "uniffi.error_types.TupleException${'$'}Oops: v1=oops")
}

try {
    oopsTuple(1u)
    throw RuntimeException("Should have failed")
} catch (e: TupleException) {
    assert(e.toString() == "uniffi.error_types.TupleException${'$'}Value: v1=1")
}

runBlocking {
    try {
        aoops()
        throw RuntimeException("Should have failed")
    } catch (e: ErrorInterface) {
        assert(e.toString() == "async-oops")
    }
}
