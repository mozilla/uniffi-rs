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

// Records
let r = TraitRecord(s: "yo", i: 0)
assert(String(reflecting: r) == "TraitRecord { s: \"yo\", i: 0 }")
assert(TraitRecord(s: "yo", i: 0) == TraitRecord(s: "yo", i: 1))
assert(TraitRecord(s: "yo", i: 0) == TraitRecord(s: "yo", i: 2))
assert(TraitRecord(s: "yo", i: 0) != TraitRecord(s: "a", i: 0))
assert(TraitRecord(s: "a", i: 1) < TraitRecord(s: "yo", i: 0))

assert(String(reflecting: UdlRecord(s: "yo", i: 1)) == "UdlRecord { s: \"yo\", i: 1 }")
assert(UdlRecord(s: "yo", i: 0) == UdlRecord(s: "yo", i: 1))
assert(UdlRecord(s: "yo", i: 0) == UdlRecord(s: "yo", i: 2))
assert(UdlRecord(s: "yo", i: 0) != UdlRecord(s: "a", i: 0))
assert(UdlRecord(s: "a", i: 1) < UdlRecord(s: "yo", i: 0))

// Enums
let e = UdlEnum.s(s: "yo")
assert(String(reflecting: e) == "S { s: \"yo\" }")
assert(UdlEnum.s(s: "yo") == UdlEnum.s(s: "yo"))
assert(UdlEnum.s(s: "yo") == UdlEnum.s(s: ""))
assert(UdlEnum.s(s: "yo") != UdlEnum.i(i: 1))

let u = TraitEnum.i(0)
assert(String(describing: u) == "TraitEnum::I(0)")
assert(String(reflecting: u) == "I(0)")
assert(u == TraitEnum.i(1))
assert(TraitEnum.s("hi") < TraitEnum.i(1))

let u_set: Set = [TraitEnum.i(0)]
assert(u_set.contains(TraitEnum.i(1)))
assert(!u_set.contains(TraitEnum.s("")))
assert(XyzEnum.xyzNone == XyzEnum.xyzNone)
assert(XyzEnum.xyzNamed(xyzValue: 0) < XyzEnum.xyzNamed(xyzValue: 1))
