import uniffi_test.Geometry;
import uniffi_test.Line;
import uniffi_test.Point;

fun main(args: Array<String>) {
  val ln = Line(Point(0,0), Point(1,2))
  println("gradient of line ${ln} is ${Geometry.gradient(ln)}")
}
