from sprites import *

s = Sprite(Point(0,1))
print(s)
print(s.get_position())
s.move_to(Point(1,2))
print(s.get_position())
s.move_by(Vector(3,-1))
print(s.get_position())
