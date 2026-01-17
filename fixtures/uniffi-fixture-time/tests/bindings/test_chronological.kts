import uniffi.chronological.*;
import kotlin.time.Duration
import kotlin.time.Duration.Companion.nanoseconds
import kotlin.time.Duration.Companion.seconds
import kotlin.time.Instant

// Test passing timestamp and duration while returning timestamp
assert(add(Instant.fromEpochSeconds(100, 100), 1.seconds + 1.nanoseconds)
        .equals(Instant.fromEpochSeconds(101, 101)))

// Test passing timestamp while returning duration
assert(diff(Instant.fromEpochSeconds(101, 101), Instant.fromEpochSeconds(100, 100))
        .equals(1.seconds + 1.nanoseconds))

// Test pre-epoch timestamps
assert(add(Instant.parse("1955-11-05T00:06:00.283000001Z"), 1.seconds + 1.nanoseconds)
        .equals(Instant.parse("1955-11-05T00:06:01.283000002Z")))

// Test exceptions are propagated
try {
        diff(Instant.fromEpochSeconds(100), Instant.fromEpochSeconds(101))
        throw RuntimeException("Should have thrown a TimeDiffError exception!")
} catch (e: ChronologicalException) {
        // It's okay!
}

// Test max Instant upper bound
assert(add(Instant.MAX, 0.seconds).equals(Instant.MAX))

// Test max Instant upper bound overflow
try {
        add(Instant.MAX, 1.seconds)
        throw RuntimeException("Should have thrown an IllegalArgumentException exception!")
} catch (e: IllegalArgumentException) {
        // It's okay!
}

// Test that rust timestamps behave like kotlin timestamps
// Unfortunately the JVM clock may be lower resolution than the Rust clock.
// Sleep for 1ms between each call, which should ensure the JVM clock ticks
// forward.
val kotlinBefore = Instant.now()
Thread.sleep(10)
val rustNow = now()
Thread.sleep(10)
val kotlinAfter = Instant.now()
assert(kotlinBefore.isBefore(rustNow))
assert(kotlinAfter.isAfter(rustNow))

// Test optional values work
assert(optional(Instant.MAX, 0.seconds))
assert(optional(null, 0.seconds) == false)
assert(optional(Instant.MAX, null) == false)
