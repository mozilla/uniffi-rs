// A handful of classes and functions to support the generated data structures.
// This would be a good candidate for isolating in its own ffi-support lib.

abstract class FFIObject(
    private val handle: AtomicLong
) {
    open fun destroy() {
        this.handle.set(0L)
    }

    internal inline fun <R> callWithHandle(block: (handle: Long) -> R) =
        this.handle.get().let { handle -> 
            if (handle != 0L) {
                block(handle)
            } else {
                throw IllegalStateException("${this.javaClass.simpleName} object has already been destroyed")
            }
        }
}

inline fun <T : FFIObject, R> T.use(block: (T) -> R) =
    try {
        block(this)
    } finally {
        try {
            this.destroy()
        } catch (e: Throwable) {
            // swallow
        }
    }
