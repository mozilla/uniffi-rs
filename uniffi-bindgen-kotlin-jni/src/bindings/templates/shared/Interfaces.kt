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
            for (arg in args) {
                when (arg) {
                    is Disposable -> arg.destroy()
                    is ArrayList<*> -> {
                        for (idx in arg.indices) {
                            val element = arg[idx]
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                    is Map<*, *> -> {
                        for (element in arg.values) {
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                    is Iterable<*> -> {
                        for (element in arg) {
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                }
            }
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

/** 
 * Placeholder object used to signal that we're constructing an interface with a FFI handle.
 *
 * This is the first argument for interface constructors that input a raw handle. It exists is that
 * so we can avoid signature conflicts when an interface has a regular constructor than inputs a
 * Long.
 *
 * */
object WithHandle

/** 
 * Used to instantiate an interface without an actual pointer, for fakes in tests, mostly.
 *
 * */
object NoHandle

{% if root.disable_java_cleaner() %}
{% include "BuiltinCleaner.kt" %}

{% else if root.enable_android_cleaner() %}
val CLEANER = android.system.SystemCleaner.cleaner()
typealias Cleanable = java.lang.ref.Cleaner.Cleanable

{% else %}
val CLEANER = java.lang.ref.Cleaner.create()
typealias Cleanable = java.lang.ref.Cleaner.Cleanable
{% endif %}
