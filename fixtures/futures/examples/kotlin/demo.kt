package my.demo

import kotlinx.coroutines.*
import kotlin.coroutines.*
import uniffi.fixture.futures.*

fun main() = runBlocking {
    launch {
        delay(100L)
        println(greet("Gordon"))
        println(alwaysReady())
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