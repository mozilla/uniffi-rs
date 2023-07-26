import uniffi.uniffi_example_futures.*
import kotlinx.coroutines.*

runBlocking {
    assert(sayAfter(10U, "Alice") == "Hello, Alice!")

    val store = Store(CoroutineScope(Dispatchers.IO))
    assert(store.loadItem() == "this was loaded from disk")
}
