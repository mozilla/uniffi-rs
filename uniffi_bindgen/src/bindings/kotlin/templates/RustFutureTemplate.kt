
typealias RustFuture = Pointer

interface RustFutureWaker: Callback {
    fun callback(env: Pointer?)
}