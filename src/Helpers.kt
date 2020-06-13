/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package mozilla.appservices.support.uniffi

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * This file contains shared helpers for accessing the compiled rust code from Kotlin.
 * It's a trimmed-down version of the helpers we currently use in application-services.
 *
 * There is a very big TODO to figure out here with regard to how megazording would work
 * in the world of uniffi, but ideally, we'd hide it behind the helpers in this file.
 */


/**
 * Contains all the boilerplate for loading a library binding given its name.
 *
 * Indirect as in, we aren't using JNA direct mapping. Eventually we'd
 * like to (it's faster), but that's a problem for another day.
 */
inline fun <reified Lib : Library> loadIndirect(
    componentName: String
): Lib {
    // XXX TODO: This will probably grow some magic for resolving megazording in future.
    // E.g. we might start by looking for the named component in `libuniffi.so` and if
    // that fails, fall back to loading it separately from `lib${componentName}.so`.
    return Native.load<Lib>(componentName, Lib::class.java)
}


/**
 * This is a mapping for the `ffi_support::ByteBuffer` struct.
 *
 * The name differs for two reasons.
 *
 * 1. To indicate that the memory this type manages is allocated from rust code,
 *    and must subsequently be freed by rust code.
 *
 * 2. To avoid confusion with java's nio ByteBuffer, which we use for
 *    passing data *to* Rust without incurring additional copies.
 *
 * # Caveats:
 *
 * 1. It is for receiving data *FROM* Rust, and not the other direction.
 *    RustBuffer doesn't expose a way to inspect its contents from Rust.
 *
 * 2. A `RustBuffer` passed into kotlin code must be freed by kotlin
 *    code *after* its contents have been completely deserialized.
 *
 *    The rust code must expose a destructor for this purpose,
 *    and it should be called in the finally block after the data
 *    is read from the `CodedInputStream` (and not before).
 *
 * 3. You almost always should use `RustBuffer.ByValue` instead
 *    of `RustBuffer`. E.g.
 *    `fun mylib_get_stuff(some: X, args: Y): RustBuffer.ByValue`
 *    for the function returning the RustBuffer, and
 *    `fun mylib_destroy_bytebuffer(bb: RustBuffer.ByValue)`.
 */
@Structure.FieldOrder("len", "data")
open class RustBuffer : Structure() {
    @JvmField var len: Long = 0
    @JvmField var data: Pointer? = null

    class ByValue : RustBuffer(), Structure.ByValue

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer(): ByteBuffer? {
        return this.data?.let {
            val buf = it.getByteBuffer(0, this.len)
            buf.order(ByteOrder.BIG_ENDIAN)
            return buf
        }
    }
}


public fun Int.Companion.deserializeItemFromRust(buf: ByteBuffer): Int {
    return buf.getInt()
}

public fun Int.serializeForRustSize(): Int {
    return 4
}

public fun Int.serializeForRustInto(buf: ByteBuffer) {
    println("SERIALIZING ${this}")
    buf.putInt(this)
}