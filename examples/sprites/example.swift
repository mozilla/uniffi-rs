import sprites

let s = Sprite(initial_position: Point(x: 0.0, y: 1.0))
print("Sprite \(s) is at position \(s.get_position())")
s.move_to(position: Point(x: 1.0, y: 2.0))
print("Sprite \(s) has moved to position \(s.get_position())")
s.move_by(direction: Vector(dx: -4.0, dy: 2.0))
print("Sprite \(s) has moved to position \(s.get_position())")


