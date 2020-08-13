import rondpoint

let dico = Dictionnaire(un: .deux, deux: false, petitNombre: 0, grosNombre: 123456789)
let copyDico = copieDictionnaire(d: dico)
assert(dico == copyDico)

assert(copieEnumeration(e: .deux) == .deux)
assert(copieEnumerations(e: [.un, .deux]) == [.un, .deux])
assert(copieCarte(c: ["1": .un, "2": .deux]) == ["1": .un, "2": .deux])

assert(switcheroo(b: false))
