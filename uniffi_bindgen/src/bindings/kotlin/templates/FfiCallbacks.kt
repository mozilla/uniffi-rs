internal interface UniffiCallbackFunction : com.sun.jna.Callback {
    fun callback(uniffiFfiBuffer: Pointer)
}

// Callback bound with a data argument
//
// This is the typical way callbacks are passed across the FFI.
//
// `data` is an opaque handle that contains some context used to execute the callback.
internal class UniffiBoundCallback(internal val callback: UniffiCallbackFunction, internal val data: Long)
