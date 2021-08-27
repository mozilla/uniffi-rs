package my.decorator;

/// MyDecorator is defined in the UDL file. It generates an interface.
/// Implementations of the interface never cross the FFI boundary, and so
/// can contain arbitrary Kotlin.
///
/// Note: MyDecorator must be in the same package as the generated bindings in order for UniFFI to see it.
class MyDecorator {
    var count = 0
    var lastString: String? = null

    fun <T> withReturn(thunk: () -> T): T = thunk()
    fun <T> stringSaver(thunk: () -> T) {
        lastString = thunk() as? String
    }
    fun <T> withCounter(thunk: () -> T): Int = thunk().let { ++count }
}
