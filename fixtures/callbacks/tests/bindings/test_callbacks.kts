/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.callbacks.*

// A bit more systematic in testing, but this time in English.
//
// 1. Pass in the callback as arguments.
// Make the callback methods use multiple aruments, with a variety of types, and
// with a variety of return types.
val rustGetters = RustGetters()
class KotlinGetters(): ForeignGetters {
    override fun getBool(v: Boolean, arg2: Boolean): Boolean = v xor arg2
    override fun getString(v: String, arg2: Boolean): String = if (arg2) "1234567890123" else v
    override fun getOption(v: String?, arg2: Boolean): String? = if (arg2) v?.uppercase() else v
    override fun getList(v: List<Int>, arg2: Boolean): List<Int> = if (arg2) v else listOf()
}

val callback = KotlinGetters()
listOf(true, false).forEach { v ->
    val flag = true
    val expected = callback.getBool(v, flag)
    val observed = rustGetters.getBool(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf(listOf(1,2), listOf(0,1)).forEach { v ->
    val flag = true
    val expected = callback.getList(v, flag)
    val observed = rustGetters.getList(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf("Hello", "world").forEach { v ->
    val flag = true
    val expected = callback.getString(v, flag)
    val observed = rustGetters.getString(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}

listOf("Some", null).forEach { v ->
    val flag = false
    val expected = callback.getOption(v, flag)
    val observed = rustGetters.getOption(callback, v, flag)
    assert(expected == observed) { "roundtripping through callback: $expected != $observed" }
}
rustGetters.destroy()

// 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
// This is crucial if we want to configure a system at startup,
// then use it without passing callbacks all the time.

class StoredKotlinStringifier: StoredForeignStringifier {
    override fun fromSimpleType(value: Int): String = "kotlin: $value"
    // We don't test this, but we're checking that the arg type is included in the minimal list of types used
    // in the UDL.
    // If this doesn't compile, then look at TypeResolver.
    override fun fromComplexType(values: List<Double?>?): String = "kotlin: $values"
}

val kotlinStringifier = StoredKotlinStringifier()
val rustStringifier = RustStringifier(kotlinStringifier)
listOf(1, 2).forEach { v ->
    val expected = kotlinStringifier.fromSimpleType(v)
    val observed = rustStringifier.fromSimpleType(v)
    assert(expected == observed) { "callback is sent on construction: $expected != $observed" }
}
rustStringifier.destroy()
