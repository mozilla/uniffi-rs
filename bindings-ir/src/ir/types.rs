/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::ir::names::*;
use serde::{Deserialize, Serialize};

/// Type used in the AST
///
/// Types can have several properties which are listed in the docstring for their variant:
///   - Primitive: primitive data like ints and floats.  Primitive types are passed by copy and
///     do not support references and can not be nullable.  Non-primitive types are passed by move
///     and support references when a move is not desirable.
///   - FFI: Can be used in FFI functions that call into the native C-API library.
///   - User: User-friendly" types that will feel familiar to developers if they are returned from
///     a bindings function.
///
/// This type system needs to be compatible with the type system of the target language, which
/// leads to two tricky issues:
///
/// # Mutability
///
/// Many languages have a concept of const vs mutable variables, but the meaning differs.  For JS
/// const means a variable can't be reassigned, but its contains can change.  For Kotlin val means
/// a variable can't be reassigned, but lists/maps can be mutated and object fields can be mutated
/// if they're declared as var.  For Rust, normal variables can't be mutated or reassigned while
/// both are valid for mutable variables.
///
/// In order to support all models, we need to distinguish all cases:
///   - `Var` declares a re-assignable variable, while `Val` declares a non-reassignable variable
///   - For both of those, the `mutable` property determines if the value inside the variable can be
///     mutated.
///   - For class fields, the `mutable` property determines if the field can be changed (both
///     the varable and the field must be declared mutable).
///
/// # Ownership
///
/// Since we're dealing with raw pointers, we need to figure out some system of ownership that
/// avoids things like use-after-free errrors. After we pass an pointer into a function, can we use
/// it or not?  The same question arises for classes/cstructs that contain a pointer field.
///
/// A related question is how to pass around list/map/string instances without copying them?  We
/// need some way to determine which piece of code is responsible for freeing the data at the end
/// of all of it.
///
/// I can think of several solutions to this:
///
/// - Wild-west style.  Copy around pointers and objects that contain them without regard to
///   ownership and make freeing the data the generated code's responsibility. This would keep the
///   AST simple, but is obviously dangerous and would make the generated code harder to reason
///   about.
///
/// - Ref-counting.  We could pass around ref-counted pointers and free things once the ref-count
///   becomes zero.  This would cover the list/map/string cases great in ref-counted languages, but
///   it wouldn't be so great if we wanted to add support for non-ref counted languages. For
///   example in C++ its faster and more idiomatic to use `vector<T>` for lists rather than
///   `shared_ptr<vector<T>>`.  Another issue is how to handle pointers being passed to Rust FFI
///   functions that consume the value, like `rust_buffer_reserve()`. That said, this is a
///   reasonable alternative that we should consider.
///
/// - Ownership system.  Use a Rust-style ownership system where we identify when we're borrowing
///   data vs when we've moving it.  This adds complexity to the AST code, but it gives a lot of
///   flexbility and I think that will lead to the nicest generated APIs (for example the C++
///   bindings can return `vector<T>`. To keep things simple, we impose the requirement that
///   references can never be stored or returned.
///
///   Right now this ownership system is just described in comments, but not actually enforced.
///   Hovewer, I believe that we can use a simple/restricted system for borrowing and that will
///   enable us to write simple borrow checker to enforce the rules.
///
///   I think it should be pretty straightforward to translate the borrow system into our target
///   languages.  Ref-counted languages can just treat a borrow as a regular reference, but they
///   will still get the benefits of the borrow checker -- for example after passing an object into
///   an FFI function that takes ownership of it, that object will still be in scope, but the
///   borrow checker can ensure that it's not used.  Other languages will need to do some
///   translation, but I think it should be easy.  In C++, passing a reference to a function will
///   be rendered as `foo` and passing an owned value will be rendered as `std::move(foo)`.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum Type {
    /// 8-bit unsigned int.
    ///
    /// Primitive/User/FFI
    UInt8,
    /// 8-bit signed int.
    ///
    /// Primitive/User/FFI
    Int8,
    /// 16-bit unsigned int.
    ///
    /// Primitive/User/FFI
    UInt16,
    /// 16-bit signed int.
    ///
    /// Primitive/User/FFI
    Int16,
    /// 32-bit unsigned int.
    ///
    /// Primitive/User/FFI
    UInt32,
    /// 32-bit signed int.
    ///
    /// Primitive/User/FFI
    Int32,
    /// 64-bit unsigned int.
    ///
    /// Primitive/User/FFI
    UInt64,
    /// 64-bit signed int.
    ///
    /// Primitive/User/FFI
    Int64,
    /// 32-bit float.
    ///
    /// Primitive/User/FFI
    Float32,
    /// 64-bit float
    ///
    /// Primitive/User/FFI
    Float64,
    /// Pointer to a Rust object.
    ///
    /// Name is a unique string that identifies the type of pointer.  Eventually we should add a
    /// static type check that functions are called with the correct type of pointers.
    ///
    /// FFI functions should be intentional about inputting a Pointer or Reference<Pointer>.  Both
    /// have the same representation, but Pointer should be used when the function consumes the
    /// underlying value and Reference<Pointer> should be used when it doesn't.
    ///
    /// FFI
    Pointer { name: String },
    /// C struct value
    ///
    /// This is basically the same as Record and some languages will use the same type for both.
    /// But for many languages, the type that can be passed across the FFI is not very
    /// user-friendly, In Kotlin, it's the difference between `com.sun.jna.Structure` and a data
    /// class.
    ///
    /// FFI
    CStruct { name: CStructName },
    /// True/false value
    ///
    /// Primitive/User
    Boolean,
    /// String value
    ///
    /// User
    String,
    /// Instance of a Class, DataClass, Enum, CStruct, BufferStream, etc.
    ///
    /// User
    Object { class: ClassName },
    /// Nullable type
    ///
    /// The inner type can not be another `Nullable`.
    ///
    /// User.  FFI when the inner type is a Pointer.
    Nullable { inner: Box<Type> },
    /// List of homogenious data
    ///
    /// User
    List { inner: Box<Type> },
    /// Key-Value map.  Keys can either be Int or String types.
    ///
    /// User
    Map { key: Box<Type>, value: Box<Type> },
    /// Reference to a type
    Reference { inner: Box<Type>, mutable: bool },
}
