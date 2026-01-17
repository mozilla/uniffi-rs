/**
 * @suppress
 */
public object FfiConverterDuration: FfiConverterRustBuffer<kotlin.time.Duration> {
    override fun read(buf: ByteBuffer): kotlin.time.Duration {
        // Type mismatch (should be u64) but we check for overflow/underflow below
        val seconds = buf.getLong()
        // Type mismatch (should be u32) but we check for overflow/underflow below
        val nanoseconds = buf.getInt().toLong()
        if (seconds < 0) {
            throw IllegalArgumentException("Duration exceeds minimum or maximum value supported by uniffi")
        }
        if (nanoseconds < 0) {
            throw IllegalArgumentException("Duration nanoseconds exceed minimum or maximum supported by uniffi")
        }
        return seconds.toDuration(kotlin.time.DurationUnit.SECONDS) + nanoseconds.toDuration(kotlin.time.DurationUnit.NANOSECONDS)
    }

    // 8 bytes for seconds, 4 bytes for nanoseconds
    override fun allocationSize(value: kotlin.time.Duration) = 12UL

    override fun write(value: kotlin.time.Duration, buf: ByteBuffer) {
        val seconds = value.inWholeSeconds
        if (seconds < 0) {
            // Rust does not support negative Durations
            throw IllegalArgumentException("Invalid duration, must be non-negative")
        }

        val nanoseconds = value.inWholeNanoseconds
        if (nanoseconds < 0) {
            // This should be impossible for valid durations
            throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
        }

        // Type mismatch (should be u64) but since Rust doesn't support negative durations we should be OK
        buf.putLong(seconds)
        // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
        buf.putInt(nanoseconds.toInt())
    }
}
