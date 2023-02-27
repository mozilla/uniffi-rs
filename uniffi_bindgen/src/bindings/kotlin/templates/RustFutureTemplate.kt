
typealias RustFuture = Pointer

interface RustFutureWaker: Callback {
    fun callback(envCStructure: RustFutureWakerEnvironmentCStructure?)
}

class RustFutureWakerEnvironment<C>(
    val rustFuture: RustFuture,
    val continuation: Continuation<C>,
    val waker: RustFutureWaker,
    val selfAsCStructure: RustFutureWakerEnvironmentCStructure,
    val coroutineScope: CoroutineScope,
)

@Structure.FieldOrder("hash")
class RustFutureWakerEnvironmentCStructure: Structure() {
    @JvmField var hash: Int = 0
}
