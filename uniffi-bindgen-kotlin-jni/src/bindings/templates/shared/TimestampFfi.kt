fun readTimestamp(cursor: uniffi.FfiBufferCursor): java.time.Instant {
    val seconds = readLong(cursor)
    val nanoseconds = readUInt(cursor)
    if (seconds >= 0) {
        return java.time.Instant.EPOCH.plus(java.time.Duration.ofSeconds(seconds, nanoseconds.toLong()))
    } else {
        return java.time.Instant.EPOCH.minus(java.time.Duration.ofSeconds(-seconds, nanoseconds.toLong()))
    }
}

fun writeTimestamp(cursor: uniffi.FfiBufferCursor, value: java.time.Instant) {
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

    writeLong(cursor, sign * epochOffset.seconds)
    writeUInt(cursor, epochOffset.nano.toUInt())
}
