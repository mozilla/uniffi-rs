import uniffi.uniffi_bindgen_tests.*
import java.time.Duration
import java.time.Instant

val now = Instant.now()
assert(roundtripSystemtime(now) == now)
assert(roundtripDuration(Duration.ofMinutes(5)) == Duration.ofMinutes(5))
