import uniffi_test.Arithmetic;
import uniffi_test.Overflow;

fun main(args: Array<String>) {
  println("2 + 3 = ${Arithmetic.add(2, 3, Overflow.SATURATING)}")
}
