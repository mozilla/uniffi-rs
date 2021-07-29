import uniffi.chronological.*;
import java.time.Duration
import java.time.Instant
import java.time.DateTimeException

// Test passing timestamp and duration while returning timestamp
assert(add(Instant.ofEpochSecond(100, 100), Duration.ofSeconds(1, 1))
        .equals(Instant.ofEpochSecond(101, 101)))

// Test passing timestamp while returning duration
assert(diff(Instant.ofEpochSecond(101, 101), Instant.ofEpochSecond(100, 100))
        .equals(Duration.ofSeconds(1, 1)))

// Test pre-epoch timestamps
assert(add(Instant.parse("1955-11-05T00:06:00.283000001Z"), Duration.ofSeconds(1, 1))
        .equals(Instant.parse("1955-11-05T00:06:01.283000002Z")))

// Test exceptions are propagated
try {
        diff(Instant.ofEpochSecond(100), Instant.ofEpochSecond(101))
        throw RuntimeException("Should have thrown a TimeDiffError exception!")
} catch (e: ChronologicalException) {
        // It's okay!
}

// Test max Instant upper bound
assert(add(Instant.MAX, Duration.ofSeconds(0)).equals(Instant.MAX))

// Test max Instant upper bound overflow
try {
        add(Instant.MAX, Duration.ofSeconds(1))
        throw RuntimeException("Should have thrown a DateTimeException exception!")
} catch (e: DateTimeException) {
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
