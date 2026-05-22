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
