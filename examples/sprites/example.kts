import uniffi.sprites.*;

val s = Sprite(Point(0.0,1.0))
println("Sprite ${s} is at position ${s.get_position()}")
s.move_to(Point(1.0, 2.0))
println("Sprite ${s} has moved to position ${s.get_position()}")
s.move_by(Vector(-4.0, 2.0))
println("Sprite ${s} has moved to position ${s.get_position()}")

