/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.proc_macro.*;

val one = makeOne(123)
assert(one.inner == 123)

val two = Two("a")
assert(takeTwo(two) == "a")

val rwb = RecordWithBytes(byteArrayOf(1,2,3).toUByteArray().toList())
assert(takeRecordWithBytes(rwb).toUByteArray().toByteArray().contentEquals(byteArrayOf(1, 2, 3)))

var obj = Object()
obj = Object.namedCtor(1u)
assert(obj.isHeavy() == MaybeBool.UNCERTAIN)

assert(enumIdentity(MaybeBool.TRUE) == MaybeBool.TRUE)

// just make sure this works / doesn't crash
val three = Three(obj)

assert(makeZero().inner == "ZERO")
assert(makeRecordWithBytes().someBytes.toUByteArray().toByteArray().contentEquals(byteArrayOf(0, 1, 2, 3, 4)))

try {
    alwaysFails()
    throw RuntimeException("alwaysFails should have thrown")
} catch (e: BasicException) {
}

obj.doStuff(5u)

try {
    obj.doStuff(0u)
    throw RuntimeException("doStuff should throw if its argument is 0")
} catch (e: FlatException) {
}


class KtTestCallbackInterface : TestCallbackInterface {
    override fun doNothing() { }

    override fun add(a: UInt, b: UInt) = a + b

    override fun optional(a: UInt?) = a ?: 0u

    override fun withBytes(rwb: RecordWithBytes) = rwb.someBytes

    override fun tryParseInt(value: String): UInt {
        if (value == "force-unexpected-error") {
            // raise an error that's not expected
            throw RuntimeException(value)
        }
        try {
            return value.toUInt()
        } catch(e: NumberFormatException) {
            throw BasicException.InvalidInput()
        }
    }

    override fun callbackHandler(o: Object): UInt {
        val v = o.takeError(BasicException.InvalidInput());
        return v
    }
}

testCallbackInterface(KtTestCallbackInterface())
