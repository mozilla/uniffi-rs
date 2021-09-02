 internal fun liftTimestamp(rbuf: RustBuffer.ByValue): java.time.Instant {
     return liftFromRustBuffer(rbuf) { buf ->
         readTimestamp(buf)
     }
 }

 internal fun readTimestamp(buf: ByteBuffer): java.time.Instant {
     val seconds = buf.getLong()
     // Type mismatch (should be u32) but we check for overflow/underflow below
     val nanoseconds = buf.getInt().toLong()
     if (nanoseconds < 0) {
         throw java.time.DateTimeException("Instant nanoseconds exceed minimum or maximum supported by uniffi")
     }
     if (seconds >= 0) {
         return java.time.Instant.EPOCH.plus(java.time.Duration.ofSeconds(seconds, nanoseconds))
     } else {
         return java.time.Instant.EPOCH.minus(java.time.Duration.ofSeconds(-seconds, nanoseconds))
     }
 }

 internal fun lowerTimestamp(v: java.time.Instant): RustBuffer.ByValue {
     return lowerIntoRustBuffer(v) { v, buf ->
         writeTimestamp(v, buf)
     }
 }

 internal fun writeTimestamp(v: java.time.Instant, buf: RustBufferBuilder) {
     var epochOffset = java.time.Duration.between(java.time.Instant.EPOCH, v)

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

     buf.putLong(sign * epochOffset.seconds)
     // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
     buf.putInt(epochOffset.nano)
 }
