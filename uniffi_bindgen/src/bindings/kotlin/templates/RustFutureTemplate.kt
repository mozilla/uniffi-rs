
typealias RustFuture = Pointer

interface RustFutureWaker: Callback {
    fun callback(envCStructure: RustFutureWakerEnvironmentCStructure?)
}

interface RustFutureWakerEnvironment<C> {
    val rustFuture: RustFuture
    val continuation: Continuation<C>
    val waker: RustFutureWaker
    val asCStructure: RustFutureWakerEnvironmentCStructure
}

@Structure.FieldOrder("hash")
class RustFutureWakerEnvironmentCStructure: Structure() {
    @JvmField var hash: Int = 0
}
