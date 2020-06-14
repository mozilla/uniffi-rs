import uniffi.arithmetic.*;

fun main(args: Array<String>) {
  println("2 + 3 = ${add(2, 3, Overflow.SATURATING)}")
}
