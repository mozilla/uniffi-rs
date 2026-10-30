// Deconstructed version of Timestamp
class {{ type_node.lowered_type_kt() }}(
    val v0: kotlin.Long,
    val v1: kotlin.Int,
)

fun {{ type_node.lower_fn_kt() }}(value: java.time.Instant): {{ type_node.lowered_type_kt() }} {
    var epochOffset = java.time.Duration.between(java.time.Instant.EPOCH, value)

    var sign = 1
    if (epochOffset.isNegative()) {
        sign = -1
        epochOffset = epochOffset.negated()
    }

    if (epochOffset.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Instant.html
        throw IllegalArgumentException("Invalid timestamp, nano value must be non-negative")
    }

    // Note: nanoseconds is a u32 in Rust, but we just return a kotlin `Int` directly without bounds
    // checking.  We just checked that it was non-negative and it should never overflow the i32 max.
    return {{ type_node.lowered_type_kt() }}(sign * epochOffset.seconds, epochOffset.nano)
}

fun {{ type_node.lift_fn_kt() }}(seconds: kotlin.Long, nanoseconds: kotlin.Int): java.time.Instant {
    if (seconds >= 0) {
        return java.time.Instant.EPOCH.plus(java.time.Duration.ofSeconds(seconds, nanoseconds.toLong()))
    } else {
        return java.time.Instant.EPOCH.minus(java.time.Duration.ofSeconds(-seconds, nanoseconds.toLong()))
    }
}

fun {{ type_node.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: java.time.Instant) {
    val lowered = {{ type_node.lower_fn_kt() }}(value)
    writeLong(buf, offset, lowered.v0)
    writeInt(buf, offset + 8, lowered.v1)
}

fun {{ type_node.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): java.time.Instant {
    return {{ type_node.lift_fn_kt() }}(
        readLong(buf, offset),
        readInt(buf, offset + 8),
    )
}
