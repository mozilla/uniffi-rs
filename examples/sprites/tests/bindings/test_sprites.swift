import sprites

let s = Sprite(initial_position: Point(x: 0, y: 1))
assert( s.get_position() == Point(x: 0, y: 1))

s.move_to(position: Point(x: 1.0, y: 2.0))
assert( s.get_position() == Point(x: 1, y: 2))

s.move_by(direction: Vector(dx: -4, dy: 2))
assert( s.get_position() == Point(x: -3, y: 4))


