fun readDuration(cursor: uniffi.FfiBufferCursor): java.time.Duration {
    // seconsd is unsigned in Rust, but signed in the JVM.
    //
    // Read the value as signed and if we detect a negative number that means we've
    // overflowed the JVM limit.
    val seconds = readLong(cursor)
    if (seconds < 0) {
        throw java.time.DateTimeException("Duration exceeds minimum or maximum value supported by uniffi")
    }
    val nanoseconds = readUInt(cursor)
    return java.time.Duration.ofSeconds(seconds, nanoseconds.toLong())
}

fun writeDuration(cursor: uniffi.FfiBufferCursor, value: java.time.Duration) {
   if (value.seconds < 0) {
        // Rust does not support negative Durations
        throw IllegalArgumentException("Invalid duration, must be non-negative")
    }

    if (value.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Duration.html
        throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
    }
    writeLong(cursor, value.seconds)
    writeInt(cursor, value.nano)
}
