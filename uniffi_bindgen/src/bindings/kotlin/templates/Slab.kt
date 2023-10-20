internal class UniffiSlab<T> {
    val slabHandle = _UniFFILib.INSTANCE.{{ ci.ffi_slab_new().name() }}()
    var lock = ReentrantReadWriteLock()
    var items = mutableListOf<T?>()

    private fun index(handle: UniffiHandle): Int = (handle and 0xFFFF).toInt()

    internal fun insert(value: T): UniffiHandle {
        val handle = _UniFFILib.INSTANCE.{{ ci.ffi_slab_insert().name() }}(slabHandle)
        if (handle < 0) {
            throw InternalException("Slab insert error")
        }
        val index = index(handle)
        return lock.writeLock().withLock {
            while (items.size <= index) {
                items.add(null)
            }
            items[index] = value
            handle
        }
    }

    internal fun get(handle: UniffiHandle): T {
        val result = _UniFFILib.INSTANCE.{{ ci.ffi_slab_check_handle().name() }}(slabHandle, handle)
        if (result < 0) {
            throw InternalException("Slab get error")
        }
        return lock.readLock().withLock { items[index(handle)]!! }
    }

    internal fun remove(handle: UniffiHandle): T {
        val result = _UniFFILib.INSTANCE.{{ ci.ffi_slab_dec_ref().name() }}(slabHandle, handle)
        if (result < 0) {
            throw InternalException("Slab remove error")
        }
        val index = index(handle)
        return lock.writeLock().withLock {
            items[index]!!.also {
                if (result == 1.toByte()) {
                    items[index] = null 
                }
            }
        }
    }
}
