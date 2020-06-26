import geometry

let ln1 = Line(start: Point(x: 0, y: 0), end: Point(x: 1,y: 2))
let ln2 = Line(start: Point(x: 1, y: 1), end: Point(x: 2,y: 2))

print("gradient of line \(ln1) is \(gradient(ln: ln1))")
print("gradient of line \(ln2) is \(gradient(ln: ln2))")

print("intersection of the two lines is \(String(describing: intersection(ln1: ln1, ln2: ln2)))")
print("intersection of a line with itself is \(String(describing: intersection(ln1: ln1,ln2: ln1)))")
