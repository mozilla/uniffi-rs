/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.time.Instant
import java.util.concurrent.*

import uniffi.coverall.*

// TODO: use an actual test runner.

// Test some_dict().
// N.B. we need to `use` here to clean up the contained `Coveralls` reference.
createSomeDict().use { d ->
    assert(d.text == "text")
    assert(d.maybeText == "maybe_text")
    assert(d.someBytes.contentEquals("some_bytes".toByteArray(Charsets.UTF_8)))
    assert(d.maybeSomeBytes.contentEquals("maybe_some_bytes".toByteArray(Charsets.UTF_8)))
    assert(d.aBool)
    assert(d.maybeABool == false)
    assert(d.unsigned8 == 1.toUByte())
    assert(d.maybeUnsigned8 == 2.toUByte())
    assert(d.unsigned16 == 3.toUShort())
    assert(d.maybeUnsigned16 == 4.toUShort())
    assert(d.unsigned64 == 18446744073709551615UL)
    assert(d.maybeUnsigned64 == 0UL)
    assert(d.signed8 == 8.toByte())
    assert(d.maybeSigned8 == 0.toByte())
    assert(d.signed64 == 9223372036854775807L)
    assert(d.maybeSigned64 == 0L)

    // floats should be "close enough".
    fun Float.almostEquals(other: Float) = Math.abs(this - other) < 0.000001
    fun Double.almostEquals(other: Double) = Math.abs(this - other) < 0.000001

    assert(d.float32.almostEquals(1.2345F))
    assert(d.maybeFloat32!!.almostEquals(22.0F/7.0F))
    assert(d.float64.almostEquals(0.0))
    assert(d.maybeFloat64!!.almostEquals(1.0))

    assert(d.coveralls!!.getName() == "some_dict")
}

createNoneDict().use { d ->
    assert(d.text == "text")
    assert(d.maybeText == null)
    assert(d.someBytes.contentEquals("some_bytes".toByteArray(Charsets.UTF_8)))
    assert(d.maybeSomeBytes == null)
    assert(d.aBool)
    assert(d.maybeABool == null)
    assert(d.unsigned8 == 1.toUByte())
    assert(d.maybeUnsigned8 == null)
    assert(d.unsigned16 == 3.toUShort())
    assert(d.maybeUnsigned16 == null)
    assert(d.unsigned64 == 18446744073709551615UL)
    assert(d.maybeUnsigned64 == null)
    assert(d.signed8 == 8.toByte())
    assert(d.maybeSigned8 == null)
    assert(d.signed64 == 9223372036854775807L)
    assert(d.maybeSigned64 == null)

    // floats should be "close enough".
    fun Float.almostEquals(other: Float) = Math.abs(this - other) < 0.000001
    fun Double.almostEquals(other: Double) = Math.abs(this - other) < 0.000001

    assert(d.float32.almostEquals(1.2345F))
    assert(d.maybeFloat32 == null)
    assert(d.float64.almostEquals(0.0))
    assert(d.maybeFloat64 == null)

    assert(d.coveralls == null)
}


// Test arcs.

Coveralls("test_arcs").use { coveralls ->
    assert(getNumAlive() == 1UL);
    // One ref held by the foreign-language code, one created for this method call.
    assert(coveralls.strongCount() == 2UL);
    assert(coveralls.getOther() == null);
    coveralls.takeOther(coveralls);
    // Should now be a new strong ref, held by the object's reference to itself.
    assert(coveralls.strongCount() == 3UL);
    // But the same number of instances.
    assert(getNumAlive() == 1UL);
    // Careful, this makes a new Kotlin object which must be separately destroyed.
    coveralls.getOther()!!.use { other ->
        // It's the same Rust object.
        assert(other.getName() == "test_arcs")
    }
    try {
        coveralls.takeOtherFallible()
        throw RuntimeException("Should have thrown an IntegerOverflow exception!")
    } catch (e: CoverallException.TooManyHoles) {
        // It's okay!
    }
    try {
        coveralls.takeOtherPanic("expected panic: with an arc!")
        throw RuntimeException("Should have thrown an InternalException!")
    } catch (e: InternalException) {
        // No problemo!
    }

    try {
        coveralls.falliblePanic("Expected panic in a fallible function!")
        throw RuntimeException("Should have thrown an InternalException")
    } catch (e: InternalException) {
        // No problemo!
    }
    coveralls.takeOther(null);
    assert(coveralls.strongCount() == 2UL);
}
assert(getNumAlive() == 0UL);

// Test return objects

Coveralls("test_return_objects").use { coveralls ->
    assert(getNumAlive() == 1UL)
    assert(coveralls.strongCount() == 2UL)
    coveralls.cloneMe().use { c2 ->
        assert(c2.getName() == coveralls.getName())
        assert(getNumAlive() == 2UL)
        assert(c2.strongCount() == 2UL)

        coveralls.takeOther(c2)
        // same number alive but `c2` has an additional ref count.
        assert(getNumAlive() == 2UL)
        assert(coveralls.strongCount() == 2UL)
        assert(c2.strongCount() == 3UL)
    }
    // Here we've dropped Kotlin's reference to `c2`, but the rust struct will not
    // be dropped as coveralls hold an `Arc<>` to it.
    assert(getNumAlive() == 2UL)
}
// Destroying `coveralls` will kill both.
assert(getNumAlive() == 0UL);

Coveralls("test_simple_errors").use { coveralls ->
    try {
        coveralls.maybeThrow(true)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: CoverallException.TooManyHoles) {
        // Expected result
        assert(e.message == "The coverall has too many holes")
    }

    try {
        coveralls.maybeThrowInto(true)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: CoverallException.TooManyHoles) {
        // Expected result
    }

    try {
        coveralls.panic("oops")
        throw RuntimeException("Expected method to throw exception")
    } catch(e: InternalException) {
        // Expected result
        assert(e.message == "oops")
    }
}

Coveralls("test_complex_errors").use { coveralls ->
    assert(coveralls.maybeThrowComplex(0) == true)

    try {
        coveralls.maybeThrowComplex(1)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: ComplexException.OsException) {
        assert(e.code == 10.toShort())
        assert(e.extendedCode == 20.toShort())
        assert(e.toString() == "uniffi.coverall.ComplexException\$OsException: code=10, extendedCode=20") {
            "Unexpected ComplexException.OsError.toString() value: ${e.toString()}"
        }
    }

    try {
        coveralls.maybeThrowComplex(2)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: ComplexException.PermissionDenied) {
        assert(e.reason == "Forbidden")
        assert(e.toString() == "uniffi.coverall.ComplexException\$PermissionDenied: reason=Forbidden") {
            "Unexpected ComplexException.PermissionDenied.toString() value: ${e.toString()}"
        }
    }

    try {
        coveralls.maybeThrowComplex(3)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: ComplexException.UnknownException) {
        assert(e.toString() == "uniffi.coverall.ComplexException\$UnknownException: ") {
            "Unexpected ComplexException.UnknownException.toString() value: ${e.toString()}"
        }
    }

    try {
        coveralls.maybeThrowComplex(4)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: InternalException) {
        // Expected result
    }
}

Coveralls("test_interfaces_in_dicts").use { coveralls ->
    coveralls.addPatch(Patch(Color.RED))
    coveralls.addRepair(
            Repair(`when`=Instant.now(), patch=Patch(Color.BLUE))
        )
    assert(coveralls.getRepairs().size == 2)
}

// Test traits implemented in Rust
makeRustGetters().let { rustGetters ->
    testGetters(rustGetters)
    testGettersFromKotlin(rustGetters)
}

fun testGettersFromKotlin(getters: Getters) {
    assert(getters.getBool(true, true) == false);
    assert(getters.getBool(true, false) == true);
    assert(getters.getBool(false, true) == true);
    assert(getters.getBool(false, false) == false);

    assert(getters.getString("hello", false) == "hello");
    assert(getters.getString("hello", true) == "HELLO");

    assert(getters.getOption("hello", true) == "HELLO");
    assert(getters.getOption("hello", false) == "hello");
    assert(getters.getOption("", true) == null);

    assert(getters.getList(listOf(1, 2, 3), true) == listOf(1, 2, 3))
    assert(getters.getList(listOf(1, 2, 3), false) == listOf<Int>())

    assert(getters.getNothing("hello") == Unit);

    try {
        getters.getString("too-many-holes", true)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: CoverallException.TooManyHoles) {
        // Expected
    }

    try {
        getters.getOption("os-error", true)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: ComplexException.OsException) {
        assert(e.code.toInt() == 100)
        assert(e.extendedCode.toInt() == 200)
    }

    try {
        getters.getOption("unknown-error", true)
        throw RuntimeException("Expected method to throw exception")
    } catch(e: ComplexException.UnknownException) {
        // Expected
    }

    try {
        getters.getString("unexpected-error", true)
    } catch(e: InternalException) {
        // Expected
    }
}

// Test NodeTrait
getTraits().let { traits ->
    assert(traits[0].name() == "node-1")
    // Note: strong counts are 1 more than you might expect, because the strongCount() method
    // holds a strong ref.
    assert(traits[0].strongCount() == 2UL)

    assert(traits[1].name() == "node-2")
    assert(traits[1].strongCount() == 2UL)

    traits[0].setParent(traits[1])
    assert(ancestorNames(traits[0]) == listOf("node-2"))
    assert(ancestorNames(traits[1]).isEmpty())
    assert(traits[1].strongCount() == 3UL)
    assert(traits[0].getParent()!!.name() == "node-2")
    traits[0].setParent(null)

    Coveralls("test_regressions").use { coveralls ->
        assert(coveralls.getStatus("success") == "status: success")
    }
}

// This tests that the UniFFI-generated scaffolding doesn't introduce any unexpected locking.
// We have one thread busy-wait for a some period of time, while a second thread repeatedly
// increments the counter and then checks if the object is still busy. The second thread should
// not be blocked on the first, and should reliably observe the first thread being busy.
// If it does not, that suggests UniFFI is accidentally serializing the two threads on access
// to the shared counter object.

ThreadsafeCounter().use { counter ->
    val executor = Executors.newFixedThreadPool(3)
    try {
        val busyWaiting: Future<Unit> = executor.submit(Callable {
            // 300 ms should be long enough for the other thread to easily finish
            // its loop, but not so long as to annoy the user with a slow test.
            counter.busyWait(300)
        })
        val incrementing: Future<Int> = executor.submit(Callable {
            var count = 0
            for (n in 1..100) {
                // We exect most iterations of this loop to run concurrently
                // with the busy-waiting thread.
                count = counter.incrementIfBusy()
            }
            count
        })

        busyWaiting.get()
        val count = incrementing.get()
        assert(count > 0) { "Counter doing the locking: incrementIfBusy=$count" }
    } finally {
        executor.shutdown()
    }
}

// This does not call Rust code.
var d = DictWithDefaults()
assert(d.name == "default-value")
assert(d.category == null)
assert(d.integer == 31UL)

d = DictWithDefaults(name = "this", category = "that", integer = 42UL)
assert(d.name == "this")
assert(d.category == "that")
assert(d.integer == 42UL)

// Test bytes
Coveralls("test_bytes").use { coveralls ->
    assert(coveralls.reverse("123".toByteArray(Charsets.UTF_8)).toString(Charsets.UTF_8) == "321")
}
