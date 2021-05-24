import org.mozilla.uniffi.example.chronological.*;
import java.time.Duration
import java.time.Instant
import java.time.DateTimeException

// Test passing timestamp and duration while returning timestamp
assert(add(Instant.ofEpochSecond(100, 100), Duration.ofSeconds(1, 1))
        .equals(Instant.ofEpochSecond(101, 101)))

// Test passing timestamp while returning duration
assert(diff(Instant.ofEpochSecond(101, 101), Instant.ofEpochSecond(100, 100))
        .equals(Duration.ofSeconds(1, 1)))

// Test exceptions are propagated
try {
        diff(Instant.ofEpochSecond(100), Instant.ofEpochSecond(101))
        throw RuntimeException("Should have thrown a TimeDiffError exception!")
} catch (e: ChronologicalErrorException) {
        // It's okay!
}

// Test unix epoch lower bound
try {
        diff(Instant.ofEpochSecond(-1), Instant.ofEpochSecond(101))
        throw RuntimeException("Should have thrown a IllegalArgumentException exception!")
} catch (e: IllegalArgumentException) {
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
val kotlinBefore = Instant.now()
val rustNow = now()
val kotlinAfter = Instant.now()

assert(kotlinBefore.isBefore(rustNow) || kotlinBefore.equals(rustNow))
assert(kotlinAfter.isAfter(rustNow) || kotlinAfter.equals(rustNow))