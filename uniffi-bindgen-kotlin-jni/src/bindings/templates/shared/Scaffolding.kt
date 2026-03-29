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
    {%- for func in package.functions %}
    @JvmStatic external fun {{ func.jni_method_name }}(uniffiBufferHandle: Long)
    {%- endfor %}
    {%- endfor %}

    // access `uniffiLibrary` to make sure the cdylib is loaded
    init {
        System.loadLibrary("{{ cdylib }}")
    }
}
