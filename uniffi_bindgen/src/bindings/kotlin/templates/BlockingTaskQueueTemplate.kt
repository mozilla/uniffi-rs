object uniffiBlockingTaskQueueClone : UniffiBlockingTaskQueueClone {
    override fun callback(handle: Long): Long  {
        val coroutineContext = uniffiBlockingTaskQueueHandleMap.get(handle)
        return uniffiBlockingTaskQueueHandleMap.insert(coroutineContext)
    }
}

object uniffiBlockingTaskQueueFree : UniffiBlockingTaskQueueFree {
    override fun callback(handle: Long) {
        uniffiBlockingTaskQueueHandleMap.remove(handle)
    }
}

internal val uniffiBlockingTaskQueueVTable = UniffiBlockingTaskQueueVTable(
    uniffiBlockingTaskQueueClone,
    uniffiBlockingTaskQueueFree,
)

public object {{ ffi_converter_name }}: FfiConverterRustBuffer<CoroutineContext> {
    override fun allocationSize(value: {{ type_name }}) = 16UL

    override fun write(value: CoroutineContext, buf: ByteBuffer) {
        // Call `write()` to make sure the data is written to the JNA backing data
        uniffiBlockingTaskQueueVTable.write()
        val handle = uniffiBlockingTaskQueueHandleMap.insert(value)
        buf.putLong(handle)
        buf.putLong(Pointer.nativeValue(uniffiBlockingTaskQueueVTable.getPointer()))
    }

    override fun read(buf: ByteBuffer): CoroutineContext {
        val handle = buf.getLong()
        val coroutineContext = uniffiBlockingTaskQueueHandleMap.remove(handle)
        // Read the VTable pointer and throw it out.  The vtable is only used by Rust and always the
        // same value.
        buf.getLong()
        return coroutineContext
    }
}

// For testing
public fun uniffiBlockingTaskQueueHandleCount() = uniffiBlockingTaskQueueHandleMap.size
