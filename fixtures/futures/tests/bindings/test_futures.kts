import uniffi.fixture.futures.*
import kotlinx.coroutines.*
import kotlin.system.*

// init UniFFI to get good measurements after that
runBlocking {
    val time = measureTimeMillis {
        alwaysReady()
    }

    println("init time: ${time}ms")
}

fun assertReturnsImmediately(actualTime: Long, testName:  String) {
    assert(actualTime <= 4) {
        "unexpected $testName time: ${actualTime}ms"
    }
}

fun assertApproximateTime(actualTime: Long, expectedTime: Int, testName:  String) {
    assert(actualTime >= expectedTime && actualTime <= expectedTime + 100) {
        "unexpected $testName time: ${actualTime}ms"
    }
}

// Test `always_ready`.
runBlocking {
    val time = measureTimeMillis {
        val result = alwaysReady()

        assert(result == true)
    }

    assertReturnsImmediately(time, "always_ready")
}

// Test `void`.
runBlocking {
    val time = measureTimeMillis {
        val result = void()

        assert(result == Unit)
    }

    assertReturnsImmediately(time, "void")
}

// Test `sleep`.
runBlocking {
    val time = measureTimeMillis {
        sleep(200U)
    }

    assertApproximateTime(time, 200, "sleep")
}

// Test sequential futures.
runBlocking {
    val time = measureTimeMillis {
        val resultAlice = sayAfter(100U, "Alice")
        val resultBob = sayAfter(200U, "Bob")

        assert(resultAlice == "Hello, Alice!")
        assert(resultBob == "Hello, Bob!")
    }

    assertApproximateTime(time, 300, "sequential future")
}

// Test concurrent futures.
runBlocking {
    val time = measureTimeMillis {
        val resultAlice = async { sayAfter(100U, "Alice") }
        val resultBob = async { sayAfter(200U, "Bob") }

        assert(resultAlice.await() == "Hello, Alice!")
        assert(resultBob.await() == "Hello, Bob!")
    }

    assertApproximateTime(time, 200, "concurrent future")
}

// Test async methods.
runBlocking {
    val megaphone = newMegaphone()
    val time = measureTimeMillis {
        val resultAlice = megaphone.sayAfter(200U, "Alice")

        assert(resultAlice == "HELLO, ALICE!")
    }

    assertApproximateTime(time, 200, "async methods")
}

// Test async method returning optional object
runBlocking {
    val megaphone = asyncMaybeNewMegaphone(true)
    assert(megaphone != null)

    val not_megaphone = asyncMaybeNewMegaphone(false)
    assert(not_megaphone == null)
}

// Test with the Tokio runtime.
runBlocking {
    val time = measureTimeMillis {
        val resultAlice = sayAfterWithTokio(200U, "Alice")

        assert(resultAlice == "Hello, Alice (with Tokio)!")
    }

    assertApproximateTime(time, 200, "with tokio runtime")
}

// Test fallible function/method.
runBlocking {
    val time1 = measureTimeMillis {
        try {
            fallibleMe(false)
            assert(true)
        } catch (exception: Exception) {
            assert(false) // should never be reached
        }
    }

    print("fallible function (with result): ${time1}ms")
    assert(time1 < 100)
    println(" ... ok")

    val time2 = measureTimeMillis {
        try {
            fallibleMe(true)
            assert(false) // should never be reached
        } catch (exception: Exception) {
            assert(true)
        }
    }

    print("fallible function (with exception): ${time2}ms")
    assert(time2 < 100)
    println(" ... ok")

    val megaphone = newMegaphone()

    val time3 = measureTimeMillis {
        try {
             megaphone.fallibleMe(false)
            assert(true)
        } catch (exception: Exception) {
            assert(false) // should never be reached
        }
    }

    print("fallible method (with result): ${time3}ms")
    assert(time3 < 100)
    println(" ... ok")

    val time4 = measureTimeMillis {
        try {
            megaphone.fallibleMe(true)
            assert(false) // should never be reached
        } catch (exception: Exception) {
            assert(true)
        }
    }

    print("fallible method (with exception): ${time4}ms")
    assert(time4 < 100)
    println(" ... ok")
}

// Test record.
runBlocking {
    val time = measureTimeMillis {
        val result = newMyRecord("foo", 42U)

        assert(result.a == "foo")
        assert(result.b == 42U)
    }

    print("record: ${time}ms")
    assert(time < 100)
    println(" ... ok")
}

// Test a broken sleep.
runBlocking {
    val time = measureTimeMillis {
        brokenSleep(100U, 0U) // calls the waker twice immediately
        sleep(100U) // wait for possible failure

        brokenSleep(100U, 100U) // calls the waker a second time after 1s
        sleep(200U) // wait for possible failure
    }

    assertApproximateTime(time, 500, "broken sleep")
}
