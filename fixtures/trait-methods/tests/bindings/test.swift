import trait_methods

let m = TraitMethods(name: "yo")
assert(String(describing: m) == "TraitMethods(yo)")
assert(String(reflecting: m) == "TraitMethods { val: \"yo\" }")

// eq
assert(m == TraitMethods(name: "yo"))
assert(m != TraitMethods(name: "foo"))

// hash
var set: Set = [TraitMethods(name: "yo")]
assert(set.contains(TraitMethods(name: "yo")))

// ord
assert(m < TraitMethods(name: "zo"))
assert(TraitMethods(name: "zo") > m)
