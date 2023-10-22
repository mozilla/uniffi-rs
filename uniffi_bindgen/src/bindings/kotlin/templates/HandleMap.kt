internal class UniffiHandleMap<T> {
    private val lock = ReentrantReadWriteLock()
    private var mapId: Long = UniffiHandleMap.nextMapId()
    private val map: MutableMap<Long, T> = mutableMapOf()
    // Note: Foreign handles are always odd
    private var keyCounter = 1L

    private fun nextKey(): Long = keyCounter.also {
        keyCounter = (keyCounter + 2L).and(0xFFFF_FFFF_FFFFL)
    }

    private fun makeHandle(key: Long): UniffiHandle = key.or(mapId)

    private fun key(handle: UniffiHandle): Long {
        if (handle.and(0x7FFF_0000_0000_0000L) != mapId) {
            throw InternalException("Handle map ID mismatch")
        }
        return handle.and(0xFFFF_FFFF_FFFFL)
    }
 
    fun newHandle(obj: T): UniffiHandle = lock.writeLock().withLock {
        val key = nextKey()
        map[key] = obj
        makeHandle(key)
    }
 
    fun get(handle: UniffiHandle) = lock.readLock().withLock {
        map[key(handle)] ?: throw InternalException("Missing key in handlemap: was the handle used after being freed?")
    }
 
    fun cloneHandle(handle: UniffiHandle): UniffiHandle = lock.writeLock().withLock {
        val obj = map[key(handle)] ?: throw InternalException("Missing key in handlemap: was the handle used after being freed?")
        val clone = nextKey()
        map[clone] = obj
        makeHandle(clone)
    }
 
    fun consumeHandle(handle: UniffiHandle): T = lock.writeLock().withLock {
        map.remove(key(handle)) ?: throw InternalException("Missing key in handlemap: was the handle used after being freed?")
    }

    companion object {
        // Generate map IDs that are likely to be unique
        private var mapIdCounter: Long = {{ ci.namespace_hash() }}.and(0x7FFF)

        // Map ID, shifted into the top 16 bits
        internal fun nextMapId(): Long = mapIdCounter.shl(48).also {
            // On Kotlin, map ids are only 15 bits to get around signed/unsigned issues
            mapIdCounter = ((mapIdCounter + 1).and(0x7FFF))
        }
    }
}

internal fun uniffiHandleIsFromRust(handle: Long): Boolean {
    return handle.and(1L) == 0L
}
