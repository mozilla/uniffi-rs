import rondpoint

let dico = Dictionnaire(un: .deux, deux: false)
let copyDico = copieDictionnaire(d: dico)
assert(dico == copyDico)

assert(copieEnumeration(e: .deux) == .deux)
assert(copieEnumerations(e: [.un, .deux]) == [.un, .deux])

assert(switcheroo(b: false))
