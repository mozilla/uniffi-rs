from geometry import *

ln1 = Line(Point(0,0), Point(1,2))
ln2 = Line(Point(1,1), Point(2,2))

print("gradient of line {} is {}".format(ln1, gradient(ln1)))
print("gradient of line {} is {}".format(ln2, gradient(ln2)))

print("intersection of the two lines is {} (but that doesn't sound right, so something must be buggy..?)".format(intersection(ln1, ln2)))
print("intersection of a line with itself is {}".format(intersection(ln1,ln1)))
