// Deconstructed version of Duration
class {{ type_node.lowered_type_kt() }}(
    val v0: kotlin.Long,
    val v1: kotlin.Int,
)

fun {{ type_node.lower_fn_kt() }}(value: java.time.Duration): {{ type_node.lowered_type_kt() }} {
   if (value.seconds < 0) {
        // Rust does not support negative Durations
        throw IllegalArgumentException("Invalid duration, must be non-negative")
    }

    if (value.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Duration.html
        throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
    }
    return {{ type_node.lowered_type_kt() }}(value.seconds, value.nano)
}

fun {{ type_node.lift_fn_kt() }}(seconds: kotlin.Long, nanoseconds: kotlin.Int): java.time.Duration {
    // Note: In Rust these are unsigned, but the FFI values are signed.  We normally would add a
    // `.toLong` call, but then we would just need to convert them back to signed to pass them to
    // `Duration.ofSeconds`.
    //
    // In practice, neither part should overflow the signed bounds so this should be okay
    return java.time.Duration.ofSeconds(seconds, nanoseconds.toLong())
}

fun {{ type_node.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: java.time.Duration) {
    val lowered = {{ type_node.lower_fn_kt() }}(value)
    writeLong(buf, offset, lowered.v0)
    writeInt(buf, offset + 8, lowered.v1)
}

fun {{ type_node.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): java.time.Duration {
    return {{ type_node.lift_fn_kt() }}(
        readLong(buf, offset),
        readInt(buf, offset + 8),
    )
}
