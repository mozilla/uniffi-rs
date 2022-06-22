/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// C functions to test against
use bindings_ir::ir::*;
use bytes::BufMut;
use once_cell::sync::OnceCell;
use std::ffi::c_void;

// Simple functions that input/output numbers
#[no_mangle]
pub extern "C" fn roundtrip_i8(value: i8) -> i8 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_i16(value: i16) -> i16 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_i32(value: i32) -> i32 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_i64(value: i64) -> i64 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_u8(value: u8) -> u8 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_u16(value: u16) -> u16 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_u32(value: u32) -> u32 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_u64(value: u64) -> u64 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_f32(value: f32) -> f32 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_f64(value: f64) -> f64 {
    value
}

#[no_mangle]
pub extern "C" fn roundtrip_bool(value: bool) -> bool {
    value
}

// Buffer functionality
static READ_BUFFER_CELL: OnceCell<Vec<u8>> = OnceCell::new();
static WRITE_BUFFER_CELL: OnceCell<Vec<u8>> = OnceCell::new();
// Create a string to test functions with.  This includes unicode to test tricky situations where
// the distrinction between number of bytes and number of codepoints matters
static STRING_BUFFER: &str = "🦊test-string🦊";

fn get_read_buffer() -> &'static Vec<u8> {
    READ_BUFFER_CELL.get_or_init(|| {
        let mut buf = Vec::new();
        // Fill up the buffer with a bunch of primitive data so that we can test reading it
        buf.put_u8(1);
        buf.put_i8(-1);
        buf.put_u16(2);
        buf.put_i16(-2);
        buf.put_u32(4);
        buf.put_i32(-4);
        buf.put_u64(8);
        buf.put_i64(-8);
        buf.put_f32(1.5);
        buf.put_f64(-0.5);
        buf.put_u64(buf.as_ptr() as u64);
        buf
    })
}

fn get_write_buffer() -> &'static Vec<u8> {
    WRITE_BUFFER_CELL.get_or_init(|| {
        let mut write_buffer = Vec::new();
        for _ in 0..get_read_buffer().len() {
            write_buffer.push(0)
        }
        write_buffer
    })
}

#[no_mangle]
pub extern "C" fn get_read_buffer_ptr() -> *const c_void {
    get_read_buffer().as_ptr() as *const c_void
}

#[no_mangle]
pub extern "C" fn get_read_buffer_size() -> i32 {
    get_read_buffer().len() as i32
}

#[no_mangle]
pub extern "C" fn is_read_buffer_ptr(ptr: *const c_void) -> u8 {
    (ptr == get_read_buffer_ptr()) as u8
}

#[no_mangle]
pub extern "C" fn get_write_buffer_ptr() -> *const c_void {
    get_write_buffer().as_ptr() as *const c_void
}

#[no_mangle]
pub extern "C" fn get_write_buffer_size() -> i32 {
    get_write_buffer().len() as i32
}

#[no_mangle]
pub extern "C" fn write_buffer_matches_read_buffer() -> u8 {
    (get_write_buffer() == get_read_buffer()) as u8
}

#[no_mangle]
pub extern "C" fn write_buffer_matches_string_buffer() -> u8 {
    match std::str::from_utf8(&get_write_buffer()[..STRING_BUFFER.len()]) {
        Ok(value) => (value == STRING_BUFFER) as u8,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn get_string_buffer_ptr() -> *const c_void {
    STRING_BUFFER.as_ptr() as *const c_void
}

#[no_mangle]
pub extern "C" fn get_string_buffer_size() -> i32 {
    STRING_BUFFER.len() as i32
}

// structs
#[repr(C)]
#[derive(Debug)]
pub struct Numbers {
    a: i32,
    b: i32,
}

#[no_mangle]
pub extern "C" fn make_numbers(a: i32, b: i32) -> Numbers {
    Numbers { a, b }
}

#[no_mangle]
pub extern "C" fn add_numbers(numbers: Numbers) -> i32 {
    numbers.a + numbers.b
}

#[no_mangle]
pub extern "C" fn add_numbers_ref(numbers: &Numbers) -> i32 {
    numbers.a + numbers.b
}

#[no_mangle]
pub extern "C" fn double_each_number(numbers: &mut Numbers) {
    numbers.a *= 2;
    numbers.b *= 2;
}

pub struct Adder {
    amount: i32,
}

static mut FREE_COUNT: u32 = 0;

#[no_mangle]
pub extern "C" fn adder_create(amount: i32) -> *mut Adder {
    Box::into_raw(Box::new(Adder { amount }))
}

#[no_mangle]
pub extern "C" fn adder_add(adder: *mut Adder, value: i32) -> i32 {
    let adder = unsafe { &*adder };
    value + adder.amount
}

#[no_mangle]
pub extern "C" fn adder_destroy(adder: *mut Adder) {
    // This causes the box to drop, freeing the memory
    unsafe {
        FREE_COUNT += 1;
        Box::from_raw(adder);
    }
}

#[no_mangle]
pub extern "C" fn adder_consume(adder: *mut Adder) {
    // Do the exact same thing as `adder_destroy`, but don't increment the free count.  This
    // simulates a function that inputs an Owned adder, transfering ownership from the bindings
    // back to Rust.
    unsafe {
        Box::from_raw(adder);
    }
}

#[no_mangle]
pub extern "C" fn adder_reset_free_count() {
    unsafe {
        FREE_COUNT = 0;
    }
}

#[no_mangle]
pub extern "C" fn adder_get_free_count() -> u32 {
    unsafe { FREE_COUNT }
}

pub(crate) fn test_module() -> Module {
    let mut module = Module::new();
    let functions = [
        ("i8", int8()),
        ("u8", uint8()),
        ("i16", int16()),
        ("u16", uint16()),
        ("i32", int32()),
        ("u32", uint32()),
        ("i64", int64()),
        ("u64", uint64()),
        ("f32", float32()),
        ("f64", float64()),
    ]
    .into_iter()
    .map(|(name, type_)| FFIFunctionDef {
        name: format!("roundtrip_{name}").into(),
        args: vec![arg("value", type_.clone())],
        return_type: type_.into(),
    })
    .chain(
        [
            // Buffers
            FFIFunctionDef {
                name: "get_read_buffer_ptr".into(),
                args: vec![],
                return_type: pointer("BufPtr").into(),
            },
            FFIFunctionDef {
                name: "get_read_buffer_size".into(),
                args: vec![],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "is_read_buffer_ptr".into(),
                args: vec![arg("ptr", pointer("BufPtr"))],
                return_type: uint8().into(),
            },
            FFIFunctionDef {
                name: "get_write_buffer_ptr".into(),
                args: vec![],
                return_type: pointer("BufPtr").into(),
            },
            FFIFunctionDef {
                name: "get_write_buffer_size".into(),
                args: vec![],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "get_string_buffer_ptr".into(),
                args: vec![],
                return_type: pointer("BufPtr").into(),
            },
            FFIFunctionDef {
                name: "get_string_buffer_size".into(),
                args: vec![],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "write_buffer_matches_read_buffer".into(),
                args: vec![],
                return_type: uint8().into(),
            },
            FFIFunctionDef {
                name: "write_buffer_matches_string_buffer".into(),
                args: vec![],
                return_type: uint8().into(),
            },
            FFIFunctionDef {
                name: "make_numbers".into(),
                args: vec![arg("a", int32()), arg("b", int32())],
                return_type: cstruct("Numbers").into(),
            },
            FFIFunctionDef {
                name: "add_numbers".into(),
                args: vec![arg("numbers", cstruct("Numbers"))],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "add_numbers_ref".into(),
                args: vec![arg("numbers", reference(cstruct("Numbers")))],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "double_each_number".into(),
                args: vec![arg("numbers", reference_mut(cstruct("Numbers")))],
                return_type: None,
            },
            FFIFunctionDef {
                name: "adder_create".into(),
                args: vec![arg("amount", int32())],
                return_type: pointer("Adder").into(),
            },
            FFIFunctionDef {
                name: "adder_add".into(),
                args: vec![arg("adder", pointer("Adder")), arg("value", int32())],
                return_type: int32().into(),
            },
            FFIFunctionDef {
                name: "adder_destroy".into(),
                args: vec![arg("adder", pointer("Adder"))],
                return_type: None,
            },
            FFIFunctionDef {
                name: "adder_consume".into(),
                args: vec![arg("adder", pointer("Adder"))],
                return_type: None,
            },
            FFIFunctionDef {
                name: "adder_reset_free_count".into(),
                args: vec![],
                return_type: None,
            },
            FFIFunctionDef {
                name: "adder_get_free_count".into(),
                args: vec![],
                return_type: uint32().into(),
            },
        ]
        .into_iter(),
    );

    module.add_buffer_stream_class("BufStream");
    module.add_native_library("bindings_ir_tests", functions);
    module.add_cstruct(CStructDef {
        name: "Numbers".into(),
        fields: vec![mut_field("a", int32()), mut_field("b", int32())],
    });
    // Define a CStruct with all possible types, just to make sure the definition is legal
    module.add_cstruct(CStructDef {
        name: "CStructWithAllTypes".into(),
        fields: vec![
            mut_field("i8", int8()),
            mut_field("u8", uint8()),
            mut_field("i16", int16()),
            mut_field("u16", uint16()),
            mut_field("i32", int32()),
            mut_field("u32", uint32()),
            mut_field("i64", int64()),
            mut_field("u64", uint64()),
            mut_field("f32", float32()),
            mut_field("f64", float64()),
            mut_field("numbers", cstruct("Numbers")),
            mut_field("pointer", pointer("BufPtr")),
            mut_field("nullable_pointer", nullable(pointer("BufPtr"))),
        ],
    });

    module
}
