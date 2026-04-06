// Initial value and increment amount for handles. 
// These ensure that Kotlin-generated handles always have the lowest bit set
private const val UNIFFI_HANDLEMAP_INITIAL = 1.toLong()
private const val UNIFFI_HANDLEMAP_DELTA = 2.toLong()

// Map handles to objects
//
// This is used pass an opaque 64-bit handle representing a foreign object to the Rust code.
//
// Callback handles are always odd, which allows us to differentiate them from Rust-generated
// handles.
internal class HandleMap<T: Any> {
    private val map = java.util.concurrent.ConcurrentHashMap<kotlin.Long, T>()
    private val counter = java.util.concurrent.atomic.AtomicLong(UNIFFI_HANDLEMAP_INITIAL)

    val size: Int
        get() = map.size

    // Insert a new object into the handle map and get a handle for it
    fun insert(obj: T): kotlin.Long {
        val handle = counter.getAndAdd(UNIFFI_HANDLEMAP_DELTA)
        map.put(handle, obj)
        return handle
    }

    // Clone a handle, creating a new one
    fun clone(handle: kotlin.Long): kotlin.Long {
        val obj = map.get(handle) ?: throw InternalException("HandleMap.clone: Invalid handle")
        return insert(obj)
    }

    // Get an object from the handle map
    fun get(handle: kotlin.Long): T {
        return map.get(handle) ?: throw InternalException("HandleMap.get: Invalid handle")
    }

    // Remove an entry from the handlemap and get the Kotlin object back
    fun remove(handle: kotlin.Long): T {
        return map.remove(handle) ?: throw InternalException("HandleMap: Invalid handle")
    }
}
