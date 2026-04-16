/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.proc_macro.*;

val one = makeOne(123)
assert(one.inner == 123)
assert(oneInnerByRef(one) == 123)
assert(one.getInnerValue() == 123)

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
assert(MaybeBool.TRUE.next() == MaybeBool.FALSE)
assert(enumIdentity(MaybeBool.TRUE).next() == MaybeBool.FALSE)

// just make sure this works / doesn't crash
val three = Three(obj)

assert(makeZero().inner == "ZERO")
assert(makeRecordWithBytes().someBytes.contentEquals(byteArrayOf(0, 1, 2, 3, 4)))
assert(join(listOf("a", "b", "c"), ":") == "a:b:c")

try {
    alwaysFails()
    throw RuntimeException("alwaysFails should have thrown")
} catch (e: BasicException) {
    assert(!e.isUnexpected())
}

obj.doStuff(5u)

try {
    obj.doStuff(0u)
    throw RuntimeException("doStuff should throw if its argument is 0")
} catch (e: FlatException) {
}

// Defaults

val recordWithDefaults = RecordWithDefaults("Test")
assert(recordWithDefaults.noDefaultString == "Test")
assert(recordWithDefaults.boolean == true)
assert(recordWithDefaults.integer == 42)
assert(recordWithDefaults.floatVar == 4.2)
assert(recordWithDefaults.vec.isEmpty())
assert(recordWithDefaults.optVec == null)
assert(recordWithDefaults.optInteger == 42)
assert(recordWithDefaults.customInteger == 42)

val recordWithImplicitDefaults = RecordWithImplicitDefaults()
assert(recordWithImplicitDefaults.boolean == false)
assert(recordWithImplicitDefaults.int8 == 0.toByte())
assert(recordWithImplicitDefaults.int16 == 0.toShort())
assert(recordWithImplicitDefaults.int32 == 0)
assert(recordWithImplicitDefaults.int64 == 0L)
assert(recordWithImplicitDefaults.uint8 == 0U.toUByte())
assert(recordWithImplicitDefaults.uint16 == 0U.toUShort())
assert(recordWithImplicitDefaults.uint32 == 0U)
assert(recordWithImplicitDefaults.uint64 == 0UL)
assert(recordWithImplicitDefaults.afloat == 0f)
assert(recordWithImplicitDefaults.adouble == 0.0)
assert(recordWithImplicitDefaults.vec.isEmpty())
assert(recordWithImplicitDefaults.map == emptyMap<String, String>())
assert(recordWithImplicitDefaults.someBytes.size == 0)
assert(recordWithImplicitDefaults.customInteger == 0)

assert(doubleWithDefault() == 42)
assert(sumWithDefault(2) == 2)
assert(sumWithDefault(2, 1) == 3)

val objWithDefaults = ObjectWithDefaults()
assert(objWithDefaults.addToNum() == 42)
assert(objWithDefaults.addToImplicitNum() == 30)
assert(objWithDefaults.addToImplicitNum(1) == 31)

// Traits

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

assert(getMixedEnum(null) == MixedEnum.Int(1))
assert(getMixedEnum(MixedEnum.None) == MixedEnum.None)
assert(getMixedEnum(MixedEnum.String("hello")) == MixedEnum.String("hello"))
assert(!getMixedEnum(MixedEnum.None).isNotNone());
assert(!MixedEnum.None.isNotNone());
assert(MixedEnum.Int(1).isNotNone());

val e = getMixedEnum(null)
if (e is MixedEnum.Int) {
    // you can destruct the enum into its bits.
    val (i) = e
    assert(i == 1L)
} else {
    assert(false)
}
val eb = MixedEnum.Both("hi", 2)
val (s, i) = eb
assert(s == "hi")
assert(i == 2L)

assert(getMixedEnum(MixedEnum.Vec(listOf("hello"))) == MixedEnum.Vec(listOf("hello")))
