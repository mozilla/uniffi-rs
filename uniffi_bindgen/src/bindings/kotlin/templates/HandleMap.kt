// Initial value and increment amount for handles. 
// These ensure that Kotlin-generated handles always have the lowest bit set
private const val UNIFFI_HANDLEMAP_INITIAL = 1.toLong()
private const val UNIFFI_HANDLEMAP_DELTA = 2.toLong()

// Map handles to objects
//
// This is used pass an opaque 64-bit handle representing a foreign object to the Rust code.
internal class UniffiHandleMap<T: Any> {
    private val map = ConcurrentHashMap<Long, T>()
    // Start 
    private val counter = java.util.concurrent.atomic.AtomicLong(UNIFFI_HANDLEMAP_INITIAL)

    val size: Int
        get() = map.size

    // Insert a new object into the handle map and get a handle for it
    fun insert(obj: T): Long {
        val handle = counter.getAndAdd(UNIFFI_HANDLEMAP_DELTA)
        map.put(handle, obj)
        return handle
    }

    // Clone a handle, creating a new one
    fun clone(handle: Long): Long {
        val obj = map.get(handle) ?: throw InternalException("UniffiHandleMap.clone: Invalid handle")
        return insert(obj)
    }

    // Get an object from the handle map
    fun get(handle: Long): T {
        return map.get(handle) ?: throw InternalException("UniffiHandleMap.get: Invalid handle")
    }

    // Remove an entry from the handlemap and get the Kotlin object back
    fun remove(handle: Long): T {
        return map.remove(handle) ?: throw InternalException("UniffiHandleMap: Invalid handle")
    }
}
