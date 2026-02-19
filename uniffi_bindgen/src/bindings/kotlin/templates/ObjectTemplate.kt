// This template implements a class for working with a Rust struct via a handle
// to the live Rust struct on the other side of the FFI.
//
// There's some subtlety here, because we have to be careful not to operate on a Rust
// struct after it has been dropped, and because we must expose a public API for freeing
// theq Kotlin wrapper object in lieu of reliable finalizers. The core requirements are:
//
//   * Each instance holds an opaque handle to the underlying Rust struct.
//     Method calls need to read this handle from the object's state and pass it in to
//     the Rust FFI.
//
//   * When an instance is no longer needed, its handle should be passed to a
//     special destructor function provided by the Rust FFI, which will drop the
//     underlying Rust struct.
//
//   * Given an instance, calling code is expected to call the special
//     `destroy` method in order to free it after use, either by calling it explicitly
//     or by using a higher-level helper like the `use` method. Failing to do so risks
//     leaking the underlying Rust struct.
//
//   * We can't assume that calling code will do the right thing, and must be prepared
//     to handle Kotlin method calls executing concurrently with or even after a call to
//     `destroy`, and to handle multiple (possibly concurrent!) calls to `destroy`.
//
//   * We must never allow Rust code to operate on the underlying Rust struct after
//     the destructor has been called, and must never call the destructor more than once.
//     Doing so may trigger memory unsafety.
//
//   * To mitigate many of the risks of leaking memory and use-after-free unsafety, a `Cleaner`
//     is implemented to call the destructor when the Kotlin object becomes unreachable.
//     This is done in a background thread. This is not a panacea, and client code should be aware that
//      1. the thread may starve if some there are objects that have poorly performing
//     `drop` methods or do significant work in their `drop` methods.
//      2. the thread is shared across the whole library. This can be tuned by using `android_cleaner = true`,
//         or `android = true` in the [`kotlin` section of the `uniffi.toml` file](https://mozilla.github.io/uniffi-rs/kotlin/configuration.html).
//
// If we try to implement this with mutual exclusion on access to the handle, there is the
// possibility of a race between a method call and a concurrent call to `destroy`:
//
//    * Thread A starts a method call, reads the value of the handle, but is interrupted
//      before it can pass the handle over the FFI to Rust.
//    * Thread B calls `destroy` and frees the underlying Rust struct.
//    * Thread A resumes, passing the already-read handle value to Rust and triggering
//      a use-after-free.
//
// One possible solution would be to use a `ReadWriteLock`, with each method call taking
// a read lock (and thus allowed to run concurrently) and the special `destroy` method
// taking a write lock (and thus blocking on live method calls). However, we aim not to
// generate methods with any hidden blocking semantics, and a `destroy` method that might
// block if called incorrectly seems to meet that bar.
//
// So, we achieve our goals by giving each instance an associated `AtomicLong` counter to track
// the number of in-flight method calls, and an `AtomicBoolean` flag to indicate whether `destroy`
// has been called. These are updated according to the following rules:
//
//    * The initial value of the counter is 1, indicating a live object with no in-flight calls.
//      The initial value for the flag is false.
//
//    * At the start of each method call, we atomically check the counter.
//      If it is 0 then the underlying Rust struct has already been destroyed and the call is aborted.
//      If it is nonzero them we atomically increment it by 1 and proceed with the method call.
//
//    * At the end of each method call, we atomically decrement and check the counter.
//      If it has reached zero then we destroy the underlying Rust struct.
//
//    * When `destroy` is called, we atomically flip the flag from false to true.
//      If the flag was already true we silently fail.
//      Otherwise we atomically decrement and check the counter.
//      If it has reached zero then we destroy the underlying Rust struct.
//
// Astute readers may observe that this all sounds very similar to the way that Rust's `Arc<T>` works,
// and indeed it is, with the addition of a flag to guard against multiple calls to `destroy`.
//
// The overall effect is that the underlying Rust struct is destroyed only when `destroy` has been
// called *and* all in-flight method calls have completed, avoiding violating any of the expectations
// of the underlying Rust code.
//
// This makes a cleaner a better alternative to _not_ calling `destroy()` as
// and when the object is finished with, but the abstraction is not perfect: if the Rust object's `drop`
// method is slow, and/or there are many objects to cleanup, and it's on a low end Android device, then the cleaner
// thread may be starved, and the app will leak memory.
//
// In this case, `destroy`ing manually may be a better solution.
//
// The cleaner can live side by side with the manual calling of `destroy`. In the order of responsiveness, uniffi objects
// with Rust peers are reclaimed:
//
// 1. By calling the `destroy` method of the object, which calls `rustObject.free()`. If that doesn't happen:
// 2. When the object becomes unreachable, AND the Cleaner thread gets to call `rustObject.free()`. If the thread is starved then:
// 3. The memory is reclaimed when the process terminates.
//
// [1] https://stackoverflow.com/questions/24376768/can-java-finalize-an-object-when-it-is-still-in-scope/24380219
//

{{ self.add_import("java.util.concurrent.atomic.AtomicBoolean") }}

{%- let obj = ci.get_object_definition(name).unwrap() %}
{%- let interface_name = self::object_interface_name(ci, obj) %}
{%- let impl_class_name = self::object_impl_name(ci, obj) %}
{%- let methods = obj.methods() %}
{%- let uniffi_trait_methods = obj.uniffi_trait_methods() %}
{%- let interface_docstring = obj.docstring() %}
{%- let is_error = ci.is_name_used_as_error(name) %}
{%- let ffi_converter_name = obj|ffi_converter_name %}

{%- include "Interface.kt" %}

{%- call kt::docstring(obj, 0) %}
{% if (is_error) %}
open class {{ impl_class_name }} : kotlin.Exception, Disposable, AutoCloseable, {{ interface_name }} {
{% else -%}
open class {{ impl_class_name }}: Disposable, AutoCloseable, {{ interface_name }}
{%- for t in obj.trait_impls() %}
, {{ self::trait_interface_name(ci, t.trait_ty)? }}
{% endfor %}
{%- if uniffi_trait_methods.ord_cmp.is_some() %}
, Comparable<{{ impl_class_name }}>
{%- endif %}
{
{%- endif %}

    @Suppress("UNUSED_PARAMETER")
    /**
     * @suppress
     */
    constructor(withHandle: UniffiWithHandle, handle: Long) {
        this.handle = handle
        this.cleanable = UniffiLib.CLEANER.register(this, UniffiCleanAction(handle))
    }

    /**
     * @suppress
     *
     * This constructor can be used to instantiate a fake object. Only used for tests. Any
     * attempt to actually use an object constructed this way will fail as there is no
     * connected Rust object.
     */
    @Suppress("UNUSED_PARAMETER")
    constructor(noHandle: NoHandle) {
        this.handle = 0
        this.cleanable = null
    }

    {%- match obj.primary_constructor() %}
    {%- when Some(cons) %}
    {%-     if cons.is_async() %}
    // Note no constructor generated for this object as it is async.
    {%-     else %}
    {%- call kt::docstring(cons, 4) %}
    constructor({% call kt::arg_list(cons, true) -%}) :
        this(UniffiWithHandle, {% call kt::to_ffi_call(cons) %})
    {%-     endif %}
    {%- when None %}
    {%- endmatch %}

    protected val handle: Long
    protected val cleanable: UniffiCleaner.Cleanable?

    private val wasDestroyed = AtomicBoolean(false)
    private val callCounter = AtomicLong(1)

    /**
     * Whether the current object has been destroyed and its reference is gone in the Rust side.
     */
    val isDestroyed: Boolean get() = wasDestroyed.get()

    override fun destroy() {
        // Only allow a single call to this method.
        // TODO: maybe we should log a warning if called more than once?
        if (this.wasDestroyed.compareAndSet(false, true)) {
            // This decrement always matches the initial count of 1 given at creation time.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable?.clean()
            }
        }
    }

    @Synchronized
    override fun close() {
        this.destroy()
    }

    internal inline fun <R> callWithHandle(block: (handle: Long) -> R): R {
        // Check and increment the call counter, to keep the object alive.
        // This needs a compare-and-set retry loop in case of concurrent updates.
        do {
            val c = this.callCounter.get()
            if (c == 0L) {
                throw IllegalStateException("${this.javaClass.simpleName} object has already been destroyed")
            }
            if (c == Long.MAX_VALUE) {
                throw IllegalStateException("${this.javaClass.simpleName} call counter would overflow")
            }
        } while (! this.callCounter.compareAndSet(c, c + 1L))
        // Now we can safely do the method call without the handle being freed concurrently.
        try {
            return block(this.uniffiCloneHandle())
        } finally {
            // This decrement always matches the increment we performed above.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable?.clean()
            }
        }
    }

    // Use a static inner class instead of a closure so as not to accidentally
    // capture `this` as part of the cleanable's action.
    private class UniffiCleanAction(private val handle: Long) : Runnable {
        override fun run() {
            if (handle == 0.toLong()) {
                // Fake object created with `NoHandle`, don't try to free.
                return;
            }
            uniffiRustCall { status ->
                UniffiLib.{{ obj.ffi_object_free().name() }}(handle, status)
            }
        }
    }

    /**
     * @suppress
     */
    fun uniffiCloneHandle(): Long {
        if (handle == 0.toLong()) {
            throw InternalException("uniffiCloneHandle() called on NoHandle object");
        }
        return uniffiRustCall() { status ->
            UniffiLib.{{ obj.ffi_object_clone().name() }}(handle, status)
        }
    }

    {% for meth in methods -%}
    {%- call kt::func_decl("override", meth, 4) %}
    {% endfor %}

    {% call kt::uniffi_trait_impls(uniffi_trait_methods) %}

    {# XXX - "companion object" confusion? How to have alternate constructors *and* be an error? #}
    {% if !obj.alternate_constructors().is_empty() -%}
    companion object {
        {% for cons in obj.alternate_constructors() -%}
        {% call kt::func_decl("", cons, 4) %}
        {% endfor %}
    }
    {% else if is_error %}
    companion object ErrorHandler : UniffiRustCallStatusErrorHandler<{{ impl_class_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ impl_class_name }} {
            // Due to some mismatches in the ffi converter mechanisms, errors are a RustBuffer.
            val bb = error_buf.asByteBuffer()
            if (bb == null) {
                throw InternalException("?")
            }
            return {{ ffi_converter_name }}.read(bb)
        }
    }
    {% else %}
    /**
     * @suppress
     */
    companion object
    {% endif %}
}

{%- if !obj.has_callback_interface() %}
{# Simple case: the interface can only be implemented in Rust #}

/**
 * @suppress
 */
public object {{ ffi_converter_name }}: FfiConverter<{{ type_name }}, Long> {
    override fun lower(value: {{ type_name }}): Long {
        return value.uniffiCloneHandle()
    }

    override fun lift(value: Long): {{ type_name }} {
        return {{ impl_class_name }}(UniffiWithHandle, value)
    }

    override fun read(buf: ByteBuffer): {{ type_name }} {
        return lift(buf.getLong())
    }

    override fun allocationSize(value: {{ type_name }}) = 8UL

    override fun write(value: {{ type_name }}, buf: ByteBuffer) {
        buf.putLong(lower(value))
    }
}
{%- else %}
{# 
 # The interface can be implemented in Rust or Kotlin
 # * Generate a callback interface implementation to handle the Kotlin side
 # * In the FfiConverter, check which side a handle came from to know how to handle correctly.
#}
{%- let vtable = obj.vtable().expect("trait interface should have a vtable") %}
{%- let vtable_methods = obj.vtable_methods() %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.kt" %}

/**
 * @suppress
 */
public object {{ ffi_converter_name }}: FfiConverter<{{ type_name }}, Long> {
    internal val handleMap = UniffiHandleMap<{{ type_name }}>()

    override fun lower(value: {{ type_name }}): Long {
        if (value is {{ impl_class_name }}) {
             // Rust-implemented object.  Clone the handle and return it
            return value.uniffiCloneHandle()
         } else {
            // Kotlin object, generate a new vtable handle and return that.
            return handleMap.insert(value)
         }
    }

    override fun lift(value: Long): {{ type_name }} {
        if ((value and 1.toLong()) == 0.toLong()) {
            // Rust-generated handle, construct a new class that uses the handle to implement the
            // interface
            return {{ impl_class_name }}(UniffiWithHandle, value)
        } else {
            // Kotlin-generated handle, get the object from the handle map
            return handleMap.remove(value)
        }
    }

    override fun read(buf: ByteBuffer): {{ type_name }} {
        return lift(buf.getLong())
    }

    override fun allocationSize(value: {{ type_name }}) = 8UL

    override fun write(value: {{ type_name }}, buf: ByteBuffer) {
        buf.putLong(lower(value))
    }
}
{%- endif %}
