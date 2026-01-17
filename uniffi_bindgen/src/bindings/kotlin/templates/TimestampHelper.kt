internal val UNIFFI_INSTANT_EPOCH = kotlin.time.Instant.fromEpochSeconds(0, 0)

/**
 * @suppress
 */
public object FfiConverterTimestamp: FfiConverterRustBuffer<kotlin.time.Instant> {
    override fun read(buf: ByteBuffer): kotlin.time.Instant {
        val seconds = buf.getLong()
        // Type mismatch (should be u32) but we check for overflow/underflow below
        val nanoseconds = buf.getInt().toLong()
        if (nanoseconds < 0) {
            throw IllegalArgumentException("Instant nanoseconds exceed minimum or maximum supported by uniffi")
        }
        return if (seconds >= 0) {
            UNIFFI_INSTANT_EPOCH + seconds.toDuration(kotlin.time.DurationUnit.SECONDS) + nanoseconds.toDuration(kotlin.time.DurationUnit.NANOSECONDS)
        } else {
            UNIFFI_INSTANT_EPOCH - ((-seconds).toDuration(kotlin.time.DurationUnit.SECONDS) + nanoseconds.toDuration(kotlin.time.DurationUnit.NANOSECONDS))
        }
    }

    // 8 bytes for seconds, 4 bytes for nanoseconds
    override fun allocationSize(value: kotlin.time.Instant) = 12UL

    override write(value: kotlin.time.Instant, buf: ByteBuffer) {
        var epochOffset: kotlin.time.Duration = value.epochSeconds.toDuration(kotlin.time.DurationUnit.SECONDS) + value.nanosecondsOfSecond.toDuration(kotlin.time.DurationUnit.NANOSECONDS)

        var sign = 1
        if (epochOffset.isNegative()) {
            sign = -1
            epochOffset = -epochOffset
        }

        val nanoseconds = epochOffset.inWholeNanoseconds
        if (nanoseconds < 0) {
            // Kotlin docs provide guarantee that Companion.nanoseconds will always be positive, so this should be impossible
            throw IllegalArgumentException("Invalid timestamp, nano value must be non-negative")
        }

        buf.putLong(sign * epochOffset.inWholeSeconds)
        // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
        buf.putInt(nanoseconds.toInt())
    }
}
