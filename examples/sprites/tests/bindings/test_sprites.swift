import sprites

let sempty = try! Sprite(initialPosition: nil)
assert( try! sempty.getPosition() == Point(x: 0, y: 0))

let s = try! Sprite(initialPosition: Point(x: 0, y: 1))
assert( try! s.getPosition() == Point(x: 0, y: 1))

try! s.moveTo(position: Point(x: 1.0, y: 2.0))
assert( try! s.getPosition() == Point(x: 1, y: 2))

try! s.moveBy(direction: Vector(dx: -4, dy: 2))
assert( try! s.getPosition() == Point(x: -3, y: 4))


