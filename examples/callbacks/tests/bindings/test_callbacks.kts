/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.callbacks.*

// 0. Simple example just to see it work.
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

val cbObject = OnCallAnsweredImpl()
val telephone = Telephone()

telephone.call(true, cbObject)
assert(cbObject.busyCount == 0) { "yesCount=${cbObject.busyCount} (should be 0)" }
assert(cbObject.yesCount == 1) { "yesCount=${cbObject.yesCount} (should be 1)" }

telephone.call(true, cbObject)
assert(cbObject.busyCount == 0) { "yesCount=${cbObject.busyCount} (should be 0)" }
assert(cbObject.yesCount == 2) { "yesCount=${cbObject.yesCount} (should be 2)" }

telephone.call(false, cbObject)
assert(cbObject.busyCount == 1) { "yesCount=${cbObject.busyCount} (should be 1)" }
assert(cbObject.yesCount == 2) { "yesCount=${cbObject.yesCount} (should be 2)" }

val cbObjet2 = OnCallAnsweredImpl()
telephone.call(true, cbObjet2)
assert(cbObjet2.busyCount == 0) { "yesCount=${cbObjet2.busyCount} (should be 0)" }
assert(cbObjet2.yesCount == 1) { "yesCount=${cbObjet2.yesCount} (should be 1)" }

telephone.destroy()

// A bit more systematic in testing, but this time in English.
// 
// 1. Pass in the callback as arguments.
// Make the callback methods use multiple aruments, with a variety of types, and 
// with a variety of return types.
val rtToRust = RoundTripperToRust()
class RoundTripperToKt(): RoundTripper {
    override fun getBool(v: Boolean, arg2: Boolean): Boolean = v xor arg2
    override fun getString(v: String, arg2: Boolean): String = if (arg2) "1234567890123" else v
    override fun getOption(v: String?, arg2: Boolean): String? = if (arg2) v?.toUpperCase() else v
    override fun getList(v: List<Int>, arg2: Boolean): List<Int> = if (arg2) v else listOf()
}

val callback = RoundTripperToKt()
listOf(true, false).forEach { v -> 
    val flag = true
    val expected = callback.getBool(v, flag)
    val observed = rtToRust.getBool(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf(listOf(1,2), listOf(0,1)).forEach { v -> 
    val flag = true
    val expected = callback.getList(v, flag)
    val observed = rtToRust.getList(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf("Hello", "world").forEach { v -> 
    val flag = true
    val expected = callback.getString(v, flag)
    val observed = rtToRust.getString(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf("Some", null).forEach { v -> 
    val flag = false
    val expected = callback.getOption(v, flag)
    val observed = rtToRust.getOption(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}
rtToRust.destroy()

// 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
// This is crucial if we want to configure a system at startup,
// then use it without passing callbacks all the time.

class StringifierImpl: Stringifier {
    override fun fromSimpleType(value: Int): String = "kotlin: $value"
    // We don't test this, but we're checking that the arg type is included in the minimal list of types used
    // in the UDL.
    // If this doesn't compile, then look at TypeResolver.
    override fun fromComplexType(values: List<Double?>?): String = "kotlin: $values"
}

val stCallback = StringifierImpl()
val st = StringUtil(stCallback)
listOf(1, 2).forEach { v -> 
    val expected = stCallback.fromSimpleType(v)
    val observed = st.fromSimpleType(v)
    assert(expected == observed) { "callback is sent on construction: $expected != $observed" }
}
st.destroy()