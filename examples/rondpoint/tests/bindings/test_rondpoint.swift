import rondpoint

let dico = Dictionnaire(un: .deux, deux: false)
let copyDico = try! copieDictionnaire(d: dico)
assert(dico == copyDico)

assert(try! copieEnumeration(e: .deux) == .deux)
assert(try! copieEnumerations(e: [.un, .deux]) == [.un, .deux])

assert(try! switcheroo(b: false))
