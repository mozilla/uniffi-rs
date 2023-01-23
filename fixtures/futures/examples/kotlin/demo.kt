package my.demo

import kotlinx.coroutines.*
import kotlin.coroutines.*
import kotlin.system.*
import uniffi.fixture.futures.*

fun main() = runBlocking {
    println("Let's start!\n")

    println("Wait 2secs before greeting you, dear public!")

    var time = measureTimeMillis {
        val result = sayAfter(2000U, "You")
        println("result: ${result}")
    }

    println("[in ${time / 1000.toDouble()}sec]")

    println("\nWouah, 'tired. Let's sleep for 3secs!")

    time = measureTimeMillis {
        sleep(3000U)
    }

    println("[in ${time / 1000.toDouble()}sec]")

    println("\nIs it really blocking? Nah. Let's greet Alice and Bob after resp. 2secs and 3secs _concurrently_!")

    time = measureTimeMillis {
        val alice = async { sayAfter(2000U, "Alice") }
        val bob = async { sayAfter(3000U, "Bob") }

        println("alice: ${alice.await()}")
        println("bob: ${bob.await()}")
    }

    println("[in ${time / 1000.toDouble()}sec]")

    println("\nSee, it took 3secs, not 5secs!")
}
