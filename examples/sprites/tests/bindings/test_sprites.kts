import uniffi.sprites.*;

val sempty = Sprite(null)
assert( sempty.getPosition() == Point(0.0, 0.0) )

val s = Sprite(Point(0.0, 1.0))
assert( s.getPosition() == Point(0.0, 1.0) )

s.moveTo(Point(1.0, 2.0))
assert( s.getPosition() == Point(1.0, 2.0) )

s.moveBy(Vector(-4.0, 2.0))
assert( s.getPosition() == Point(-3.0, 4.0) )
