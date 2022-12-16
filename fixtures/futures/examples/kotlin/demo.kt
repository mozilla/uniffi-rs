package my.demo

import kotlinx.coroutines.*
import kotlin.coroutines.*
import uniffi.fixture.futures.*

fun main() = runBlocking {
    launch {
        println("greet(\"Gordon\"): ${greet("Gordon")}")
        println("alwaysReady: ${alwaysReady()}")
        println("say: ${say()}")
        println("sayAfter: ${sayAfter(2.toUByte(), "Alice")}")
        println("bbb")
    }
    println("aaa")
}

suspend fun foo(): Boolean = suspendCoroutine<Boolean> { continuation ->
    // do something
    continuation.resume(true)
}



/*


// or suspendCancellableCoroutine<bool> { continuation -> … }

*/