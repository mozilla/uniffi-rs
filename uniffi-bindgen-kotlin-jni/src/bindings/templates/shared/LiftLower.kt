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
