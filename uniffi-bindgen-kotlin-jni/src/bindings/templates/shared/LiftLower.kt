fun liftUByte(v: kotlin.Byte): kotlin.UByte = v.toUByte()
fun lowerUByte(v: kotlin.UByte): kotlin.Byte = v.toByte()

fun liftByte(v: kotlin.Byte): kotlin.Byte = v
fun lowerByte(v: kotlin.Byte): kotlin.Byte = v

fun liftUShort(v: kotlin.Short): kotlin.UShort = v.toUShort()
fun lowerUShort(v: kotlin.UShort): kotlin.Short = v.toShort()

fun liftShort(v: kotlin.Short): kotlin.Short = v
fun lowerShort(v: kotlin.Short): kotlin.Short = v

fun liftUInt(v: kotlin.Int): kotlin.UInt = v.toUInt()
fun lowerUInt(v: kotlin.UInt): kotlin.Int = v.toInt()

fun liftInt(v: kotlin.Int): kotlin.Int = v
fun lowerInt(v: kotlin.Int): kotlin.Int = v

fun liftULong(v: kotlin.Long): kotlin.ULong = v.toULong()
fun lowerULong(v: kotlin.ULong): kotlin.Long = v.toLong()

fun liftLong(v: kotlin.Long): kotlin.Long = v
fun lowerLong(v: kotlin.Long): kotlin.Long = v

fun liftFloat(v: kotlin.Float): kotlin.Float = v
fun lowerFloat(v: kotlin.Float): kotlin.Float = v

fun liftDouble(v: kotlin.Double): kotlin.Double = v
fun lowerDouble(v: kotlin.Double): kotlin.Double = v

fun liftBoolean(v: kotlin.Boolean): kotlin.Boolean = v
fun lowerBoolean(v: kotlin.Boolean): kotlin.Boolean = v

fun liftString(v: kotlin.String?): kotlin.String = v!!
fun lowerString(v: kotlin.String): kotlin.String? = v

fun liftBytes(v: kotlin.ByteArray?): kotlin.ByteArray = v!!
fun lowerBytes(v: kotlin.ByteArray): kotlin.ByteArray? = v

// Optimized path for passing byte references from Kotlin -> Rust.
// These are `&[u8]` on Rust and `ByteBuffer` on Kotlin.
fun lowerBytesRef(v: java.nio.ByteBuffer): java.nio.ByteBuffer? {
    if (!v.isDirect()) {
        throw kotlin.IllegalArgumentException("Non-direct byte buffer")
    }
    return v
}

fun liftVecByte(value: kotlin.ByteArray?): kotlin.collections.List<kotlin.Byte> = value!!.toList()
fun lowerVecByte(value: kotlin.collections.List<kotlin.Byte>): kotlin.ByteArray? = value.toByteArray()

fun liftVecUByte(value: kotlin.ByteArray?): kotlin.collections.List<kotlin.UByte> = value!!.asUByteArray().toList()
fun lowerVecUByte(value: kotlin.collections.List<kotlin.UByte>): kotlin.ByteArray = value.toUByteArray().asByteArray()

fun liftVecShort(value: kotlin.ShortArray?): kotlin.collections.List<kotlin.Short> = value!!.toList()
fun lowerVecShort(value: kotlin.collections.List<kotlin.Short>): kotlin.ShortArray? = value.toShortArray()

fun liftVecUShort(value: kotlin.ShortArray?): kotlin.collections.List<kotlin.UShort> = value!!.asUShortArray().toList()
fun lowerVecUShort(value: kotlin.collections.List<kotlin.UShort>): kotlin.ShortArray = value.toUShortArray().asShortArray()

fun liftVecInt(value: kotlin.IntArray?): kotlin.collections.List<kotlin.Int> = value!!.toList()
fun lowerVecInt(value: kotlin.collections.List<kotlin.Int>): kotlin.IntArray? = value.toIntArray()

fun liftVecUInt(value: kotlin.IntArray?): kotlin.collections.List<kotlin.UInt> = value!!.asUIntArray().toList()
fun lowerVecUInt(value: kotlin.collections.List<kotlin.UInt>): kotlin.IntArray = value.toUIntArray().asIntArray()

fun liftVecLong(value: kotlin.LongArray?): kotlin.collections.List<kotlin.Long> = value!!.toList()
fun lowerVecLong(value: kotlin.collections.List<kotlin.Long>): kotlin.LongArray? = value.toLongArray()

fun liftVecULong(value: kotlin.LongArray?): kotlin.collections.List<kotlin.ULong> = value!!.asULongArray().toList()
fun lowerVecULong(value: kotlin.collections.List<kotlin.ULong>): kotlin.LongArray = value.toULongArray().asLongArray()

fun liftVecFloat(value: kotlin.FloatArray?): kotlin.collections.List<kotlin.Float> = value!!.toList()
fun lowerVecFloat(value: kotlin.collections.List<kotlin.Float>): kotlin.FloatArray? = value.toFloatArray()

fun liftVecDouble(value: kotlin.DoubleArray?): kotlin.collections.List<kotlin.Double> = value!!.toList()
fun lowerVecDouble(value: kotlin.collections.List<kotlin.Double>): kotlin.DoubleArray? = value.toDoubleArray()

fun liftOptionUByte(v: kotlin.Long): kotlin.UByte? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toUByte() }
}
fun lowerOptionUByte(v: kotlin.UByte?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionByte(v: kotlin.Long): kotlin.Byte? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toByte() }
}
fun lowerOptionByte(v: kotlin.Byte?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionUShort(v: kotlin.Long): kotlin.UShort? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toUShort() }
}
fun lowerOptionUShort(v: kotlin.UShort?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionShort(v: kotlin.Long): kotlin.Short? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toShort() }
}
fun lowerOptionShort(v: kotlin.Short?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionUInt(v: kotlin.Long): kotlin.UInt? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toUInt() }
}
fun lowerOptionUInt(v: kotlin.UInt?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionInt(v: kotlin.Long): kotlin.Int? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v.toInt() }
}
fun lowerOptionInt(v: kotlin.Int?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { v.toLong() }
}

fun liftOptionBoolean(v: kotlin.Long): kotlin.Boolean? {
    return if (v == kotlin.Long.MAX_VALUE) { null } else { v == 1L }
}
fun lowerOptionBoolean(v: kotlin.Boolean?): kotlin.Long {
    return if (v == null) { kotlin.Long.MAX_VALUE } else { if (v) { 1 } else { 0 } }
}

fun liftOptionString(v: kotlin.String?): kotlin.String? = v
fun lowerOptionString(v: kotlin.String?): kotlin.String? = v


fun liftOptionFloat(v: kotlin.Int): kotlin.Float? {
    return if (v == 0xFFFF_FFFF.toInt()) {
        null
    } else {
        kotlin.Float.fromBits(v)
    }
}

fun lowerOptionFloat(v: kotlin.Float?): kotlin.Int {
    return if (v == null) {
        0xFFFF_FFFF.toInt()
    } else {
        val bits = v.toRawBits()
        if (bits == 0xFFFF_FFFF.toInt()) {
            // The float was encoded using our special-cased NaN value.
            // Convert it to the "preferred" NaN value
            0xFFC0_0000.toInt()
        } else {
            bits
        }
    }
}

fun liftOptionDouble(v: kotlin.Long): kotlin.Double? {
    return if (v.toULong() == 0xFFFF_FFFF_FFFF_FFFFuL) {
        null
    } else {
        kotlin.Double.fromBits(v)
    }
}

fun lowerOptionDouble(v: kotlin.Double?): kotlin.Long {
    return if (v == null) {
        0xFFFF_FFFF_FFFF_FFFFuL.toLong()
    } else {
        val bits = v.toRawBits()
        if (bits.toULong() == 0xFFFF_FFFF_FFFF_FFFFuL) {
            // The float was encoded using our special-cased NaN value.
            // Convert it to the "preferred" NaN value
            0xFFF8_0000
        } else {
            bits
        }
    }
}
