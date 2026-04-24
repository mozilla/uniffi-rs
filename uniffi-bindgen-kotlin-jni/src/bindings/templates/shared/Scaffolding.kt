object Scaffolding {
    @JvmStatic external fun ffiBufferNew(): Long
    @JvmStatic external fun miniBufferNext(endPtr: Long, size: Long): Long
    @JvmStatic external fun ffiBufferFree(ptr: Long)
    @JvmStatic external fun readByte(ptr: Long): Byte
    @JvmStatic external fun readShort(ptr: Long): Short
    @JvmStatic external fun readInt(ptr: Long): Int
    @JvmStatic external fun readLong(ptr: Long): Long
    @JvmStatic external fun readFloat(ptr: Long): Float
    @JvmStatic external fun readDouble(ptr: Long): Double
    @JvmStatic external fun readString(ptr: Long): String
    @JvmStatic external fun writeByte(ptr: Long, value: Byte)
    @JvmStatic external fun writeShort(ptr: Long, value: Short)
    @JvmStatic external fun writeInt(ptr: Long, value: Int)
    @JvmStatic external fun writeLong(ptr: Long, value: Long)
    @JvmStatic external fun writeFloat(ptr: Long, value: Float)
    @JvmStatic external fun writeDouble(ptr: Long, value: Double)
    @JvmStatic external fun writeString(ptr: Long, value: String)

    {%- for package in root.packages %}
    {%- for scaffolding_function in package.scaffolding_functions %}
    {% if !scaffolding_function.callable.is_async %}
    @JvmStatic external fun {{ scaffolding_function.jni_method_name }}(uniffiBufferHandle: Long)
    {% else %}
    @JvmStatic external fun {{ scaffolding_function.jni_method_name }}(uniffiBufferHandle: Long): Long
    {% endif %}
    {%- endfor %}

    {%- for cls in package.classes() %}
    {%- if !cls.imp.is_trait_interface() %}
    @JvmStatic external fun {{ cls.jni_free_name() }}(handle: Long)
    @JvmStatic external fun {{ cls.jni_addref_name() }}(handle: Long)
    {%- else %}
    @JvmStatic external fun {{ cls.jni_free_name() }}(handle: Long, handle2: Long)
    @JvmStatic external fun {{ cls.jni_addref_name() }}(handle: Long, handle2: Long)
    {%- endif %}
    {%- endfor  %}
    {%- endfor  %}

    @JvmStatic external fun uniffiRustFuturePoll(
        rustFuture: Long,
        continuation: kotlin.coroutines.Continuation<Int>,
    ): Int
    @JvmStatic external fun uniffiRustFutureFree(rustFuture: Long)
    @JvmStatic external fun uniffiRustFutureCancel(rustFuture: Long)

    @JvmStatic external fun uniffiKotlinFutureComplete(kotlinFuture: Long, resultCode: Int)

    // access `uniffiLibrary` to make sure the cdylib is loaded
    init {
        System.loadLibrary("{{ cdylib }}")
    }
}
