import uniffi.geometry.*;

val ln1 = Line(Point(0.0,0.0), Point(1.0,2.0))
val ln2 = Line(Point(1.0,1.0), Point(2.0,2.0))
println("gradient of line ${ln1} is ${gradient(ln1)}")
println("gradient of line ${ln2} is ${gradient(ln2)}")
println("intersection of the two lines is ${intersection(ln1,ln2)}")
println("intersection of line with itself is ${intersection(ln1,ln1)}")
