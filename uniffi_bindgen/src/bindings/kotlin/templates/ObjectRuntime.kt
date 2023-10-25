{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.atomic.AtomicBoolean") }}
// Interface implemented by anything that can contain an object reference.
//
// Such types expose a `destroy()` method that must be called to cleanly
// dispose of the contained objects. Failure to call this method may result
// in memory leaks.
//
// The easiest way to ensure this method is called is to use the `.use`
// helper method to execute a block and destroy the object at the end.
interface Disposable {
    fun destroy()
    companion object {
        fun destroy(vararg args: Any?) {
            args.filterIsInstance<Disposable>()
                .forEach(Disposable::destroy)
        }
    }
}

inline fun <T : Disposable?, R> T.use(block: (T) -> R) =
    try {
        block(this)
    } finally {
        try {
            // N.B. our implementation is on the nullable type `Disposable?`.
            this?.destroy()
        } catch (e: Throwable) {
            // swallow
        }
    }

// The base class for all UniFFI Object types.
//
// This class provides core operations for working with the Rust handle to the live Rust struct on
// the other side of the FFI.
abstract class FFIObject(): Disposable, AutoCloseable {
    private val wasDestroyed = AtomicBoolean(false)

    open protected fun freeRustArcPtr() {
        // To be overridden in subclasses.
    }

    override fun destroy() {
        if (this.wasDestroyed.compareAndSet(false, true)) {
            this.freeRustArcPtr()
        }
    }

    @Synchronized
    override fun close() {
        this.destroy()
    }
}
