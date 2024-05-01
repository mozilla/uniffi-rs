from remote_types import *

testLogger = LogSink("SomeFile")
testLogger.log(LogLevel.INFO, "Hello world")

# Test the error handling
try:
    LogSink("")
    raise RuntimeError("Constructor should have thrown")
except AnyhowError:
    pass
