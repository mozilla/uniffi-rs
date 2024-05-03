import remote_types.*

val testLogger = LogSink("SomeFile")
testLogger.log(LogLevel.INFO, "Hello world")

// Test the error handling
try {
    LogSink("")
    throw RuntimeException("Constructor should have thrown")
} catch (e: AnyhowException) {
    // Expected
}
