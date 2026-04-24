const val UNIFFI_RUST_FUTURE_POLL_AGAIN = 0
const val UNIFFI_RUST_FUTURE_CANCELLED = 1
const val UNIFFI_RUST_FUTURE_COMPLETE = 2
const val UNIFFI_RUST_FUTURE_ERROR = 3
const val UNIFFI_RUST_FUTURE_FAILED = 4

const val UNIFFI_KOTLIN_FUTURE_OK = 0
const val UNIFFI_KOTLIN_FUTURE_ERR = 1

fun uniffiContinuationResume(continuation: kotlin.coroutines.Continuation<Int>) {
    continuation.resumeWith(Result.success(UNIFFI_RUST_FUTURE_POLL_AGAIN))
}

suspend fun awaitFuture(rustFuture: Long): Int {
    try {
        while(true) {
            val continuationResult = kotlin.coroutines.suspendCoroutine<Int> { continuation ->
                val pollResult = Scaffolding.uniffiRustFuturePoll(rustFuture, continuation)
                if (pollResult != UNIFFI_RUST_FUTURE_POLL_AGAIN) {
                    continuation.resumeWith(Result.success(pollResult));
                }
            }
            if (continuationResult != UNIFFI_RUST_FUTURE_POLL_AGAIN) {
                return continuationResult;
            }
        }
    } finally {
        Scaffolding.uniffiRustFutureFree(rustFuture)
    }
}
