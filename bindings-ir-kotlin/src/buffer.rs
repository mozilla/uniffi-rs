/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use bindings_ir::Renderer;
use tera::Result;

pub fn setup_renderer(renderer: &mut Renderer) -> Result<()> {
    renderer.add_ast_templates([
        (
            "BufferStreamDef",
            include_str!("templates/BufferStreamDef.kt"),
        ),
        ("BufStreamCreate", "{{ name }}({{ pointer }}, {{ size}})"),
        ("BufStreamIntoPointer", "{{ buf }}.ptr"),
        ("BufStreamPos", "{{ buf }}.byteBuf.position()"),
        ("BufStreamSetPos", "{{ buf }}.byteBuf.position({{ pos }})"),
        ("BufStreamSize", "{{ buf }}.size"),
        ("BufStreamReadInt8", "{{ buf }}.byteBuf.get()"),
        ("BufStreamReadUInt8", "{{ buf }}.byteBuf.get().toUByte()"),
        ("BufStreamReadInt16", "{{ buf }}.byteBuf.getShort()"),
        (
            "BufStreamReadUInt16",
            "{{ buf }}.byteBuf.getShort().toUShort()",
        ),
        ("BufStreamReadInt32", "{{ buf }}.byteBuf.getInt()"),
        ("BufStreamReadUInt32", "{{ buf }}.byteBuf.getInt().toUInt()"),
        ("BufStreamReadInt64", "{{ buf }}.byteBuf.getLong()"),
        (
            "BufStreamReadUInt64",
            "{{ buf }}.byteBuf.getLong().toULong()",
        ),
        ("BufStreamReadFloat32", "{{ buf }}.byteBuf.getFloat()"),
        ("BufStreamReadFloat64", "{{ buf }}.byteBuf.getDouble()"),
        (
            "BufStreamReadPointer",
            "com.sun.jna.Pointer({{ buf }}.byteBuf.getLong())",
        ),
        ("BufStreamReadString", "{{ buf }}.readString({{ size }})"),
        ("BufStreamWriteInt8", "{{ buf }}.byteBuf.put({{ value }})"),
        (
            "BufStreamWriteUInt8",
            "{{ buf }}.byteBuf.put({{ value }}.toByte())",
        ),
        (
            "BufStreamWriteInt16",
            "{{ buf }}.byteBuf.putShort({{ value }})",
        ),
        (
            "BufStreamWriteUInt16",
            "{{ buf }}.byteBuf.putShort({{ value }}.toShort())",
        ),
        (
            "BufStreamWriteInt32",
            "{{ buf }}.byteBuf.putInt({{ value }})",
        ),
        (
            "BufStreamWriteUInt32",
            "{{ buf }}.byteBuf.putInt({{ value }}.toInt())",
        ),
        (
            "BufStreamWriteInt64",
            "{{ buf }}.byteBuf.putLong({{ value }})",
        ),
        (
            "BufStreamWriteUInt64",
            "{{ buf }}.byteBuf.putLong({{ value }}.toLong())",
        ),
        (
            "BufStreamWriteFloat32",
            "{{ buf }}.byteBuf.putFloat({{ value }})",
        ),
        (
            "BufStreamWriteFloat64",
            "{{ buf }}.byteBuf.putDouble({{ value }})",
        ),
        (
            "BufStreamWritePointer",
            "{{ buf }}.byteBuf.putLong(com.sun.jna.Pointer.nativeValue({{ value }}))",
        ),
        ("BufStreamWriteString", "{{ buf }}.writeString({{ value }})"),
    ])
}
