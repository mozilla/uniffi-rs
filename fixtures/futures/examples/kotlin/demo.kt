package my.demo

import kotlinx.coroutines.*
import kotlin.coroutines.*
import kotlin.system.*
import uniffi.fixture.futures.*

fun main() = runBlocking {
    val time1 = measureTimeMillis {
        val alice = sayAfter(2.toUByte(), "Alice")
        val bob = sayAfter(3.toUByte(), "Bob")

        println("alice: ${alice}")
        println("bob: ${bob}")
    }

    println("time: ${time1}")

    val time2 = measureTimeMillis {
        val alice = async { sayAfter(2.toUByte(), "Alice") }
        val bob = async { sayAfter(2.toUByte(), "Bob") }

        println("alice: ${alice.await()}")
        println("bob: ${bob.await()}")
    }

    println("time: ${time2}")
}
