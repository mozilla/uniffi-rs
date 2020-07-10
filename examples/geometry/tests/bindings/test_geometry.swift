import geometry

let ln1 = Line(start: Point(x: 0, y: 0), end: Point(x: 1, y: 2))
let ln2 = Line(start: Point(x: 1, y: 1), end: Point(x: 2, y: 2))

assert(gradient(ln: ln1) == 2.0)
assert(gradient(ln: ln2) == 1.0)

assert(intersection(ln1: ln1, ln2: ln2) == Point(x: 0, y: 0))
assert(intersection(ln1: ln1, ln2: ln1) == nil)
