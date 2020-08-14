import uniffi.rondpoint.*

val dico = Dictionnaire(Enumeration.DEUX, true, 0, 123456789)
val copyDico = copieDictionnaire(dico)
assert(dico == copyDico)

assert(copieEnumeration(Enumeration.DEUX) == Enumeration.DEUX)
assert(copieEnumerations(listOf(Enumeration.UN, Enumeration.DEUX)) == listOf(Enumeration.UN, Enumeration.DEUX))
assert(copieCarte(mapOf("1" to Enumeration.UN, "2" to Enumeration.DEUX)) == mapOf("1" to Enumeration.UN, "2" to Enumeration.DEUX))

assert(switcheroo(false))
