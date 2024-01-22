/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.proc_macro.*;

val one = makeOne(123)
assert(one.inner == 123)
assert(oneInnerByRef(one) == 123)

val two = Two("a")
assert(takeTwo(two) == "a")

val rwb = RecordWithBytes(byteArrayOf(1,2,3))
assert(takeRecordWithBytes(rwb).contentEquals(byteArrayOf(1, 2, 3)))

var obj = Object()
obj = Object.namedCtor(1u)
assert(obj.isHeavy() == MaybeBool.UNCERTAIN)
var obj2 = Object()
assert(obj.isOtherHeavy(obj2) == MaybeBool.UNCERTAIN)

assert(enumIdentity(MaybeBool.TRUE) == MaybeBool.TRUE)

// just make sure this works / doesn't crash
val three = Three(obj)

assert(makeZero().inner == "ZERO")
assert(makeRecordWithBytes().someBytes.contentEquals(byteArrayOf(0, 1, 2, 3, 4)))
assert(join(listOf("a", "b", "c"), ":") == "a:b:c")

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

val traitImpl = obj.getTrait(null)
assert(traitImpl.concatStrings("foo", "bar") == "foobar")
assert(obj.getTrait(traitImpl).concatStrings("foo", "bar") == "foobar")
assert(concatStringsByRef(traitImpl, "foo", "bar") == "foobar")

val traitImpl2 = obj.getTraitWithForeign(null)
assert(traitImpl2.name() == "RustTraitImpl")
assert(obj.getTraitWithForeign(traitImpl2).name() == "RustTraitImpl")


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

    override fun getOtherCallbackInterface() = KtTestCallbackInterface2()
}

class KtTestCallbackInterface2 : OtherCallbackInterface {
    override fun multiply(a: UInt, b: UInt) = a * b
}

callCallbackInterface(KtTestCallbackInterface())
