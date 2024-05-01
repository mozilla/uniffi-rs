import remote_types

let testLogger = try! LogSink(path: "SomeFile")
testLogger.log(level: LogLevel.info, msg: "Hello world")

// Test the error handling
do {
    let _ = try LogSink(path: "")
    fatalError("Constructor should have thrown")
} catch is AnyhowError {
    // Expected
}
