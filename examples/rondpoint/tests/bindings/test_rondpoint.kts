import uniffi.rondpoint.*

val dico = Dictionnaire(Enumeration.DEUX, true, 0, 123456789)
val copyDico = copieDictionnaire(dico)
assert(dico == copyDico)

assert(copieEnumeration(Enumeration.DEUX) == Enumeration.DEUX)
assert(copieEnumerations(listOf(Enumeration.UN, Enumeration.DEUX)) == listOf(Enumeration.UN, Enumeration.DEUX))
assert(copieCarte(mapOf("1" to Enumeration.UN, "2" to Enumeration.DEUX)) == mapOf("1" to Enumeration.UN, "2" to Enumeration.DEUX))

assert(switcheroo(false))

// Test the roundtrip across the FFI.
// This shows that the values we send come back in exactly the same state as we sent them.
// i.e. it shows that lowering from kotlin and lifting into rust is symmetrical with 
//      lowering from rust and lifting into kotlin.
val rt = Retourneur()

fun <T> List<T>.affirmAllerRetour(fn: (T) -> T) {
    this.forEach { v ->
        assert(fn.invoke(v) == v) { "$fn($v)" }
    }
}

// Booleans
listOf(true, false).affirmAllerRetour(rt::identiqueBoolean)

// Bytes.
listOf(Byte.MIN_VALUE, Byte.MAX_VALUE).affirmAllerRetour(rt::identiqueI8)
listOf(0x00, 0xFF).map { it.toByte() }.affirmAllerRetour(rt::identiqueU8)

// Shorts
listOf(Short.MIN_VALUE, Short.MAX_VALUE).affirmAllerRetour(rt::identiqueI16)
listOf(0x0000, 0xFFFF).map { it.toShort() }.affirmAllerRetour(rt::identiqueU16)

// Ints
listOf(0, 1, -1, Int.MIN_VALUE, Int.MAX_VALUE).affirmAllerRetour(rt::identiqueI32)
listOf(0x00000000, 0xFFFFFFFF).map { it.toInt() }.affirmAllerRetour(rt::identiqueU32)

// Longs
listOf(0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE).affirmAllerRetour(rt::identiqueI64)
listOf(0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE).affirmAllerRetour(rt::identiqueU64)

// Floats
listOf(0.0F, 0.5F, 0.25F, Float.MIN_VALUE, Float.MAX_VALUE).affirmAllerRetour(rt::identiqueFloat)

// Doubles
listOf(0.0, 1.0, Double.MIN_VALUE, Double.MAX_VALUE).affirmAllerRetour(rt::identiqueDouble)

// Strings
listOf("", "abc", "été", "ښي لاس ته لوستلو لوستل", "😻emoji 👨‍👧‍👦multi-emoji, 🇨🇭a flag, a canal, panama")
    .affirmAllerRetour(rt::identiqueString)

// Test one way across the FFI.
//
// We send one representation of a value to lib.rs, and it transforms it into another, a string.
// lib.rs sends the string back, and then we compare here in kotlin.
//
// This shows that the values are transformed into strings the same way in both kotlin and rust.
// i.e. if we assume that the string return works (we test this assumption elsewhere)
//      we show that lowering from kotlin and lifting into rust has values that both kotlin and rust
//      both stringify in the same way. i.e. the same values.
//
// If we roundtripping proves the symmetry of our lowering/lifting from here to rust, and lowering/lifting from rust t here,
// and this convinces us that lowering/lifting from here to rust is correct, then 
// together, we've shown the correctness of the return leg.
val st = Stringifier()

typealias StringyEquals<T> = (observed: String, expected: T) -> Boolean
fun <T> List<T>.affirmEnchaine(
    fn: (T) -> String,
    equals: StringyEquals<T> = { obs, exp -> obs == exp.toString() }
) {
    this.forEach { exp ->
        val obs = fn.invoke(exp)
        assert(equals(obs, exp)) { "$fn($exp): observed=$obs, expected=$exp" }
    }
}

// Test the effigacy of the string transport from rust. If this fails, but everything else 
// works, then things are very weird.
val wellKnown = st.wellKnownString("kotlin")
assert("uniffi 💚 kotlin!" == wellKnown) { "wellKnownString 'uniffi 💚 kotlin!' == '$wellKnown'" }

// NB. Numbers are all signed in kotlin. This makes roundtripping of unsigned numbers tricky to show. 
// Uniffi does not generate unsigned types for kotlin, but the work tracked is 
// in https://github.com/mozilla/uniffi-rs/issues/249. Tests using unsigned types are 
// commented out for now.

// Booleans
listOf(true, false).affirmEnchaine(st::toStringBoolean)

// Bytes.
listOf(Byte.MIN_VALUE, Byte.MAX_VALUE).affirmEnchaine(st::toStringI8)
// listOf(0x00, 0xFF).map { it.toByte() }.affirmEnchaine(st::toStringU8)

// Shorts
listOf(Short.MIN_VALUE, Short.MAX_VALUE).affirmEnchaine(st::toStringI16)
// listOf(0x0000, 0xFFFF).map { it.toShort() }.affirmEnchaine(st::toStringU16)

// Ints
listOf(0, 1, -1, Int.MIN_VALUE, Int.MAX_VALUE).affirmEnchaine(st::toStringI32)
// listOf(0x00000000, 0xFFFFFFFF).map { it.toInt() }.affirmEnchaine(st::toStringU32)

// Longs
listOf(0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE).affirmEnchaine(st::toStringI64)
// listOf(0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE).affirmEnchaine(st::toStringU64)

// Floats
// MIN_VAUE is 1.4E-45. Accuracy and formatting get weird at small sizes.
listOf(0.0F, 1.0F, -1.0F, Float.MIN_VALUE, Float.MAX_VALUE).affirmEnchaine(st::toStringFloat) { s, n -> s.toFloat() == n }

// Doubles
// MIN_VALUE is 4.9E-324. Accuracy and formatting get weird at small sizes.
listOf(0.0, 1.0, -1.0, Double.MIN_VALUE, Double.MAX_VALUE).affirmEnchaine(st::toStringDouble)  { s, n -> s.toDouble() == n }