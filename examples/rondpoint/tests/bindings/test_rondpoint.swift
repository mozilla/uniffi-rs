import rondpoint

let dico = Dictionnaire(un: .deux, deux: false, petitNombre: 0, grosNombre: 123456789)
let copyDico = copieDictionnaire(d: dico)
assert(dico == copyDico)

assert(copieEnumeration(e: .deux) == .deux)
assert(copieEnumerations(e: [.un, .deux]) == [.un, .deux])
assert(copieCarte(c: ["1": .un, "2": .deux]) == ["1": .un, "2": .deux])

assert(switcheroo(b: false))

// Test the roundtrip across the FFI.
// This shows that the values we send come back in exactly the same state as we sent them.
// i.e. it shows that lowering from swift and lifting into rust is symmetrical with 
//      lowering from rust and lifting into swift.
let rt = Retourneur()

// Booleans
[true, false].affirmAllerRetour(rt.identiqueBoolean)

// Bytes.
[.min, .max].affirmAllerRetour(rt.identiqueI8)
[0x00, 0xFF].map { $0 as UInt8 }.affirmAllerRetour(rt.identiqueU8)

// Shorts
[.min, .max].affirmAllerRetour(rt.identiqueI16)
[0x0000, 0xFFFF].map { $0 as UInt16 }.affirmAllerRetour(rt.identiqueU16)

// Ints
[0, 1, -1, .min, .max].affirmAllerRetour(rt.identiqueI32)
[0x00000000, 0xFFFFFFFF].map { $0 as UInt32 }.affirmAllerRetour(rt.identiqueU32)

// Longs
[.zero, 1, -1, .min, .max].affirmAllerRetour(rt.identiqueI64)
[.zero, 1, .min, .max].affirmAllerRetour(rt.identiqueU64)

// Floats
[.zero, 1, 0.25, .leastNonzeroMagnitude, .greatestFiniteMagnitude].affirmAllerRetour(rt.identiqueFloat)

// Doubles
[0.0, 1.0, .leastNonzeroMagnitude, .greatestFiniteMagnitude].affirmAllerRetour(rt.identiqueDouble)

// Strings
["", "abc", "null\0byte", "été", "ښي لاس ته لوستلو لوستل", "😻emoji 👨‍👧‍👦multi-emoji, 🇨🇭a flag, a canal, panama"]
    .affirmAllerRetour(rt.identiqueString)

// Test one way across the FFI.
//
// We send one representation of a value to lib.rs, and it transforms it into another, a string.
// lib.rs sends the string back, and then we compare here in swift.
//
// This shows that the values are transformed into strings the same way in both swift and rust.
// i.e. if we assume that the string return works (we test this assumption elsewhere)
//      we show that lowering from swift and lifting into rust has values that both swift and rust
//      both stringify in the same way. i.e. the same values.
//
// If we roundtripping proves the symmetry of our lowering/lifting from here to rust, and lowering/lifting from rust t here,
// and this convinces us that lowering/lifting from here to rust is correct, then 
// together, we've shown the correctness of the return leg.
let st = Stringifier()

// Test the effigacy of the string transport from rust. If this fails, but everything else 
// works, then things are very weird.
let wellKnown = st.wellKnownString(value: "swift")
assert("uniffi 💚 swift!" == wellKnown, "wellKnownString 'uniffi 💚 swift!' == '\(wellKnown)'")

// Booleans
[true, false].affirmEnchaine(st.toStringBoolean)

// Bytes.
[.min, .max].affirmEnchaine(st.toStringI8)
[.min, .max].affirmEnchaine(st.toStringU8)

// Shorts
[.min, .max].affirmEnchaine(st.toStringI16)
[.min, .max].affirmEnchaine(st.toStringU16)

// Ints
[0, 1, -1, .min, .max].affirmEnchaine(st.toStringI32)
[0, 1, .min, .max].affirmEnchaine(st.toStringU32)

// Longs
[.zero, 1, -1, .min, .max].affirmEnchaine(st.toStringI64)
[.zero, 1, .min, .max].affirmEnchaine(st.toStringU64)

// Floats
[.zero, 1, -1, .leastNonzeroMagnitude, .greatestFiniteMagnitude].affirmEnchaine(st.toStringFloat) { Float.init($0) == $1 }

// Doubles
[.zero, 1, -1, .leastNonzeroMagnitude, .greatestFiniteMagnitude].affirmEnchaine(st.toStringDouble) { Double.init($0) == $1 }

// Some extension functions for testing the results of roundtripping and stringifying
extension Array where Element: Equatable {
    static func defaultEquals(_ observed: String, expected: Element) -> Bool {
        let exp = "\(expected)"
        return observed == exp
    }

    func affirmEnchaine(_ fn: (Element) -> String, equals: (String, Element) -> Bool = defaultEquals) {
        self.forEach { v in
            let obs = fn(v)
            assert(equals(obs, v), "toString_\(type(of:v))(\(v)): observed=\(obs), expected=\(v)")
        }
    }

    func affirmAllerRetour(_ fn: (Element) -> Element) {
        self.forEach { v in
            assert(fn(v) == v, "identique_\(type(of:v))(\(v))")
        }
    }
}
