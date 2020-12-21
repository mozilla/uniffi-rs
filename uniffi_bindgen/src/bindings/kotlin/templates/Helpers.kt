// A handful of classes and functions to support the generated data structures.
// This would be a good candidate for isolating in its own ffi-support lib.

abstract class FFIObject(
    private val pointer: AtomicReference<Pointer?>
) {
    open fun destroy() {
        this.pointer.set(null)
    }

    internal inline fun <R> callWithPointer(block: (pointer: Pointer) -> R) =
        this.pointer.get().let { pointer ->
            if (pointer != null) {
                block(pointer)
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
