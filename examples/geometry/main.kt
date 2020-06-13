import uniffi_test.Geometry;
import uniffi_test.Line;
import uniffi_test.Point;

fun main(args: Array<String>) {
  val ln1 = Line(Point(0,0), Point(1,2))
  val ln2 = Line(Point(1,1), Point(2,2))
  println("gradient of line ${ln1} is ${Geometry.gradient(ln1)}")
  println("gradient of line ${ln2} is ${Geometry.gradient(ln2)}")
  println("intersection of the two lines is ${Geometry.intersection(ln1,ln2)}")
}
