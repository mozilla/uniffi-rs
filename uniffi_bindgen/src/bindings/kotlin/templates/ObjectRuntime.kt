{{- self.add_import("java.lang.ref.Cleaner") }}
{%- if config.android_cleaner() %}{{- self.add_import("android.system.SystemCleaner") }}{%- endif %}
{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.atomic.AtomicBoolean") }}

// The base class for all UniFFI Object types.
//
// This class provides core operations for working with the Rust `Arc<T>` pointer to
// the live Rust struct on the other side of the FFI.
//
// There's some subtlety here, because we have to be careful not to operate on a Rust
// struct after it has been dropped, and because we must expose a public API for freeing
// the Kotlin wrapper object in lieu of reliable finalizers. The core requirements are:
//
//   * Each `FFIObject` instance holds an opaque pointer to the underlying Rust struct.
//     Method calls need to read this pointer from the object's state and pass it in to
//     the Rust FFI.
//
//   * When an `FFIObject` is no longer needed, its pointer should be passed to a
//     special destructor function provided by the Rust FFI, which will drop the
//     underlying Rust struct.
//
//   * Given an `FFIObject` instance, calling code is expected to call the special
//     `destroy` method in order to free it after use, either by calling it explicitly
//     or by using a higher-level helper like the `use` method. Failing to do so will
//     leak the underlying Rust struct.
//
//   * We can't assume that calling code will do the right thing, and must be prepared
//     to handle Kotlin method calls executing concurrently with or even after a call to
//     `destroy`, and to handle multiple (possibly concurrent!) calls to `destroy`.
//
//   * We must never allow Rust code to operate on the underlying Rust struct after
//     the destructor has been called, and must never call the destructor more than once.
//     Doing so may trigger memory unsafety.
//
// If we try to implement this with mutual exclusion on access to the pointer, there is the
// possibility of a race between a method call and a concurrent call to `destroy`:
//
//    * Thread A starts a method call, reads the value of the pointer, but is interrupted
//      before it can pass the pointer over the FFI to Rust.
//    * Thread B calls `destroy` and frees the underlying Rust struct.
//    * Thread A resumes, passing the already-read pointer value to Rust and triggering
//      a use-after-free.
//
// One possible solution would be to use a `ReadWriteLock`, with each method call taking
// a read lock (and thus allowed to run concurrently) and the special `destroy` method
// taking a write lock (and thus blocking on live method calls). However, we aim not to
// generate methods with any hidden blocking semantics, and a `destroy` method that might
// block if called incorrectly seems to meet that bar.
//
// So, we achieve our goals by giving each `FFIObject` an associated `AtomicLong` counter to track
// the number of in-flight method calls, and an `AtomicBoolean` flag to indicate whether `destroy`
// has been called. These are updated according to the following rules:
//
//    * The initial value of the counter is 1, indicating a live object with no in-flight calls.
//      The initial value for the flag is false.
//
//    * At the start of each method call, we atomically check the counter.
//      If it is 0 then the underlying Rust struct has already been destroyed and the call is aborted.
//      If it is nonzero them we atomically increment it by 1 and proceed with the method call.
//
//    * At the end of each method call, we atomically decrement and check the counter.
//      If it has reached zero then we destroy the underlying Rust struct.
//
//    * When `destroy` is called, we atomically flip the flag from false to true.
//      If the flag was already true we silently fail.
//      Otherwise we atomically decrement and check the counter.
//      If it has reached zero then we destroy the underlying Rust struct.
//
// Astute readers may observe that this all sounds very similar to the way that Rust's `Arc<T>` works,
// and indeed it is, with the addition of a flag to guard against multiple calls to `destroy`.
//
// The overall effect is that the underlying Rust struct is destroyed only when `destroy` has been
// called *and* all in-flight method calls have completed, avoiding violating any of the expectations
// of the underlying Rust code.
//
// In the future we may be able to replace some of this with automatic finalization logic, such as using
// the new "Cleaner" functionality in Java 9. The above scheme has been designed to work even if `destroy` is
// invoked by garbage-collection machinery rather than by calling code (which by the way, it's apparently also
// possible for the JVM to finalize an object while there is an in-flight call to one of its methods [1],
// so there would still be some complexity here).
//
// A cleaner makes a better alternative to _not_ calling `destroy()` as
// and when the object is finished with, but the abstraction is not perfect: if the Rust object's `drop`
// method is slow, and/or there are many objects to cleanup, and it's on a low end Android device, then the cleaner
// thread may be starved, and the app will leak memory.
//
// In this case, `destroy`ing manually may be a better solution.
//
// The cleaner can live side by side with the manual calling of `destroy`. In the order of preference, uniffi objects
// with Rust peers can be reclaimed:
//
// 1. By calling the `destroy` method of the object, which calls `rustObject.free()`. If that doesn't happen:
// 2. When the object becomes unreachable, AND the Cleaner thread gets to call `rustObject.free()`. If the thread is starved then:
// 3. The memory is reclaimed when the process terminates.
//
// [1] https://stackoverflow.com/questions/24376768/can-java-finalize-an-object-when-it-is-still-in-scope/24380219
//
abstract class FFIObject: Disposable, AutoCloseable {

    constructor(pointer: Pointer) {
        this.pointer = pointer
    }

    /**
     * This constructor can be used to instantiate a fake object.
     *
     * **WARNING: Any object instantiated with this constructor cannot be passed to an actual Rust-backed object.**
     * Since there isn't a backing [Pointer] the FFI lower functions will crash.
     * @param noPointer Placeholder value so we can have a constructor separate from the default empty one that may be
     *   implemented for classes extending [FFIObject].
     */
    @Suppress("UNUSED_PARAMETER")
    constructor(noPointer: NoPointer) {
        this.pointer = null
    }

    protected val pointer: Pointer?
    protected abstract val cleanable: Cleaner.Cleanable

    private val wasDestroyed = AtomicBoolean(false)
    private val callCounter = AtomicLong(1)

    open fun uniffiClonePointer(): Pointer {
        // Overridden by generated subclasses, the default method exists to allow users to manually
        // implement the interface
        throw RuntimeException("uniffiClonePointer not implemented")
    }

    override fun destroy() {
        // Only allow a single call to this method.
        // TODO: maybe we should log a warning if called more than once?
        if (this.wasDestroyed.compareAndSet(false, true)) {
            // This decrement always matches the initial count of 1 given at creation time.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable.clean()
            }
        }
    }

    @Synchronized
    override fun close() {
        this.destroy()
    }

    internal inline fun <R> callWithPointer(block: (ptr: Pointer) -> R): R {
        // Check and increment the call counter, to keep the object alive.
        // This needs a compare-and-set retry loop in case of concurrent updates.
        do {
            val c = this.callCounter.get()
            if (c == 0L) {
                throw IllegalStateException("${this.javaClass.simpleName} object has already been destroyed")
            }
            if (c == Long.MAX_VALUE) {
                throw IllegalStateException("${this.javaClass.simpleName} call counter would overflow")
            }
        } while (! this.callCounter.compareAndSet(c, c + 1L))
        // Now we can safely do the method call without the pointer being freed concurrently.
        try {
            return block(this.uniffiClonePointer())
        } finally {
            // This decrement always matches the increment we performed above.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable.clean()
            }
        }
    }
}

/** Used to instantiate a [FFIObject] without an actual pointer, for fakes in tests, mostly. */
object NoPointer
