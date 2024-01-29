// Handle from a UniffiHandleMap
internal typealias UniffiHandle = Long

// Map handles to objects
//
// This is used pass an opaque 64-bit handle representing a foreign object to the Rust code.
internal class UniffiHandleMap<T: Any> {
    private val map = ConcurrentHashMap<UniffiHandle, T>()
    private val counter = java.util.concurrent.atomic.AtomicLong(0)

    val size: Int
        get() = map.size

    // Insert a new object into the handle map and get a handle for it
    fun insert(obj: T): UniffiHandle {
        val handle = counter.getAndAdd(1)
        map.put(handle, obj)
        return handle
    }

    // Get an object from the handle map
    fun get(handle: UniffiHandle): T {
        return map.get(handle) ?: throw InternalException("UniffiHandleMap.get: Invalid handle")
    }

    // Remove an entry from the handlemap and get the Kotlin object back
    fun remove(handle: UniffiHandle): T {
        return map.remove(handle) ?: throw InternalException("UniffiHandleMap: Invalid handle")
    }
}
