import trait_methods

let m = TraitMethods(name: "yo")
assert(String(describing: m) == "TraitMethods(yo)")
assert(String(reflecting: m) == "TraitMethods { val: \"yo\" }")

// eq
assert(m == TraitMethods(name: "yo"))

// hash
var set: Set = [TraitMethods(name: "yo")]
assert(set.contains(TraitMethods(name: "yo")))
