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

// Test `always_ready`.
runBlocking {
    val time = measureTimeMillis {
        val result = alwaysReady()
        
        assert(result == true)
    }

    print("always_ready: ${time}ms")
    assert(time < 4)
    println(" ... ok")
}

// Test `sleep`.
runBlocking {
    val time = measureTimeMillis {
        sleep(2U)
    }
    
    print("sleep: ${time}ms")
    assert(time > 2000 && time < 2100)
    println(" ... ok")
}

// Test sequential futures.
runBlocking {
    val time = measureTimeMillis {
        val resultAlice = sayAfter(1U, "Alice")
        val resultBob = sayAfter(2U, "Bob")
        
        assert(resultAlice == "Hello, Alice!")
        assert(resultBob == "Hello, Bob!")
    }
    
    print("sequential futures: ${time}ms")
    assert(time > 3000 && time < 3100)
    println(" ... ok")
}

// Test concurrent futures.
runBlocking {
    val time = measureTimeMillis {
        val resultAlice = async { sayAfter(1U, "Alice") }
        val resultBob = async { sayAfter(2U, "Bob") }
        
        assert(resultAlice.await() == "Hello, Alice!")
        assert(resultBob.await() == "Hello, Bob!")
    }
    
    print("concurrent futures: ${time}ms")
    assert(time > 2000 && time < 2100)
    println(" ... ok")
}

// Test async methods.
runBlocking {
    val megaphone = newMegaphone()
    val time = measureTimeMillis {
        val resultAlice = megaphone.sayAfter(2U, "Alice")
        
        assert(resultAlice == "HELLO, ALICE!")
    }
    
    print("async methods: ${time}ms")
    assert(time > 2000 && time < 2100)
    println(" ... ok")
}