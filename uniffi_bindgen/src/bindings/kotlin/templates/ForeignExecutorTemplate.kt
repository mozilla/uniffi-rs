{{ self.add_import("kotlinx.coroutines.CoroutineScope") }}
{{ self.add_import("kotlinx.coroutines.delay") }}
{{ self.add_import("kotlinx.coroutines.isActive") }}
{{ self.add_import("kotlinx.coroutines.launch") }}

internal const val UNIFFI_RUST_TASK_CALLBACK_SUCCESS = 0.toByte()
internal const val UNIFFI_RUST_TASK_CALLBACK_CANCELLED = 1.toByte()
internal const val UNIFFI_FOREIGN_EXECUTOR_CALLBACK_SUCCESS = 0.toByte()
internal const val UNIFFI_FOREIGN_EXECUTOR_CALLBACK_CANCELLED = 1.toByte()
internal const val UNIFFI_FOREIGN_EXECUTOR_CALLBACK_ERROR = 2.toByte()

// Callback function to execute a Rust task.  The Kotlin code schedules these in a coroutine then
// invokes them.
internal interface UniFfiRustTaskCallback : com.sun.jna.Callback {
    fun callback(rustTaskData: Pointer?, statusCode: Byte)
}

internal object UniFfiForeignExecutorCallback : com.sun.jna.Callback {
    fun callback(handle: UniffiHandle, delayMs: Int, rustTask: UniFfiRustTaskCallback?, rustTaskData: Pointer?) : Byte {
        if (rustTask == null) {
            FfiConverterForeignExecutor.drop(handle)
            return UNIFFI_FOREIGN_EXECUTOR_CALLBACK_SUCCESS
        } else {
            val coroutineScope = FfiConverterForeignExecutor.lift(handle)
            if (coroutineScope.isActive) {
                val job = coroutineScope.launch {
                    if (delayMs > 0) {
                        delay(delayMs.toLong())
                    }
                    rustTask.callback(rustTaskData, UNIFFI_RUST_TASK_CALLBACK_SUCCESS)
                }
                job.invokeOnCompletion { cause ->
                    if (cause != null) {
                        rustTask.callback(rustTaskData, UNIFFI_RUST_TASK_CALLBACK_CANCELLED)
                    }
                }
                return UNIFFI_FOREIGN_EXECUTOR_CALLBACK_SUCCESS
            } else {
                return UNIFFI_FOREIGN_EXECUTOR_CALLBACK_CANCELLED
            }
        }
    }
}

public object FfiConverterForeignExecutor: FfiConverter<CoroutineScope, UniffiHandle> {
    internal val slab = UniffiSlab<CoroutineScope>()

    internal fun drop(handle: UniffiHandle) {
        slab.remove(handle)
    }

    internal fun register(lib: _UniFFILib) {
        {%- match ci.ffi_foreign_executor_callback_set() %}
        {%- when Some with (fn) %}
        lib.{{ fn.name() }}(UniFfiForeignExecutorCallback)
        {%- when None %}
        {#- No foreign executor, we don't set anything #}
        {% endmatch %}
    }

    override fun allocationSize(value: CoroutineScope) = 8

    override fun lift(value: UniffiHandle): CoroutineScope {
        return slab.get(value)
    }

    override fun read(buf: ByteBuffer): CoroutineScope {
        return lift(buf.getLong())
    }

    override fun lower(value: CoroutineScope): UniffiHandle {
        return slab.insert(value)
    }

    override fun write(value: CoroutineScope, buf: ByteBuffer) {
        buf.putLong(lower(value))
    }
}
