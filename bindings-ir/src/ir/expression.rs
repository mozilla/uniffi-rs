/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{names::*, Statement, Type};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum Expression {
    /// Identifier for a variable in the current scope.  Evaluates to the value of the variable
    Ident {
        name: VarName,
    },
    /// Get a value from a field of a Class, DataClass, CStruct
    ///
    /// For non-copy types, this will return a reference.
    Get {
        container: Box<Expression>,
        field: FieldName,
    },
    /// Get a reference of the current object.  Can only be used in methods.
    This,
    /// Call a function
    Call {
        name: FunctionName,
        /// Must match the types from the corresponding Function
        values: Vec<Expression>,
    },
    /// Call a method on an object
    MethodCall {
        obj: Box<Expression>,
        /// Name of the method
        name: FunctionName,
        /// Must match the types from the corresponding Method
        values: Vec<Expression>,
    },
    /// Call a static method on a class
    StaticMethodCall {
        class: ClassName,
        /// Name of the method
        name: FunctionName,
        /// Must match the types from the corresponding Method
        values: Vec<Expression>,
    },
    /// Call an FFI function
    FFICall {
        name: String,
        /// Must match the types from the corresponding FFIFunction
        values: Vec<Expression>,
    },
    /// Get a reference to a variable
    ///
    /// This is only valid as a call argument.
    Ref {
        name: VarName,
        mutable: bool,
    },
    /// Create a Nullable value from a regular value
    Some {
        inner: Box<Expression>,
    },
    /// Unwrap a nullable value, raising a runtime exception if it's null
    Unwrap {
        nullable: Box<Expression>,
        message: Box<Expression>,
    },
    /// Convert an object into a Rust value
    ///
    /// This runs the `into_rust` method for the object, which transfers ownership of the object
    /// into Rust.  This expression is only valid as an argument of an FFI function call.  After
    /// this runs, the object will no longer be considered owned by the bindings code.  It can no
    /// longer be used and it's destructor will not run.
    ///
    /// The object variable must be declared mutable.
    IntoRust {
        obj: Box<Expression>,
    },
    /// Create a new Class instance
    ClassCreate {
        name: ClassName,
        /// Values for each field in the class
        values: Vec<Expression>,
    },
    /// Create a new DataClass instance
    DataClassCreate {
        name: ClassName,
        /// Must match the types from the field list or contructor args
        values: Vec<Expression>,
    },
    /// Create a new CStruct instance
    CStructCreate {
        name: String,
        /// Must match the types from the field list or contructor args
        values: Vec<Expression>,
    },
    /// Create an enum variant
    EnumCreate {
        name: ClassName,
        variant: ClassName,
        /// Values must match the fields from the variant fields
        values: Vec<Expression>,
    },
    /// Create an exception
    ExceptionCreate {
        name: ClassName,
        values: Vec<Expression>,
    },
    /// Get the size of a pointer in bytes
    PointerSize,
    /// Cast to an Int8 value
    ///
    /// Casting a value that doesn't fit into an Int8 is undefined behavior.
    CastInt8 {
        value: Box<Expression>,
    },
    /// Cast to an Int16 value
    ///
    /// Casting a value that doesn't fit into an Int16 is undefined behavior.
    CastInt16 {
        value: Box<Expression>,
    },
    /// Cast to an Int32 value
    ///
    /// Casting a value that doesn't fit into an Int32 is undefined behavior.
    CastInt32 {
        value: Box<Expression>,
    },
    /// Cast to an Int64 value
    ///
    /// Casting a value that doesn't fit into an Int64 is undefined behavior.
    CastInt64 {
        value: Box<Expression>,
    },
    /// Cast to an UInt8 value
    ///
    /// Casting a value that doesn't fit into an UInt8 is undefined behavior.
    CastUInt8 {
        value: Box<Expression>,
    },
    /// Cast to an UInt16 value
    ///
    /// Casting a value that doesn't fit into an UInt16 is undefined behavior.
    CastUInt16 {
        value: Box<Expression>,
    },
    /// Cast to an UInt32 value
    ///
    /// Casting a value that doesn't fit into an UInt32 is undefined behavior.
    CastUInt32 {
        value: Box<Expression>,
    },
    /// Cast to an UInt64 value
    ///
    /// Casting a value that doesn't fit into an UInt64 is undefined behavior.
    CastUInt64 {
        value: Box<Expression>,
    },
    /// (T, T) -> Boolean, where T is any numeric type including FFI types
    Eq {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (T, T) -> Boolean, where T is any numeric type including FFI types
    Gt {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (T, T) -> Boolean, where T is any numeric type including FFI types
    Lt {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (T, T) -> Boolean, where T is any numeric type including FFI types
    Ge {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (T, T) -> Boolean, where T is any numeric type including FFI types
    Le {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Boolean, Boolean) -> Boolean
    And {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Boolean, Boolean) -> Boolean
    Or {
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Boolean) -> Boolean
    Not {
        value: Box<Expression>,
    },
    /// (Int, Int) -> Int
    Add {
        /// Integer type being operated on, this is needed becaause Kotlin/Java does some weird
        /// things with integer arithmetic, automatically casting things to a different type.
        ///
        /// It's unfortunate that users need to specify the type, I'm hoping that we can avoid that
        /// by doing some static analysis on the code that calculates this.
        #[serde(rename = "type")]
        type_: Type,
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Int, Int) -> Int
    Sub {
        #[serde(rename = "type")]
        type_: Type,
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Int, Int) -> Int
    Mul {
        #[serde(rename = "type")]
        type_: Type,
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// (Int, Int) -> Int
    Div {
        #[serde(rename = "type")]
        type_: Type,
        first: Box<Expression>,
        second: Box<Expression>,
    },
    /// Check if a value is an instance of a class
    IsInstance {
        value: Box<Expression>,
        class: ClassName,
    },
    /// Get a lower bound on the number of bytes needed to store a string, when encoded as UTF-8
    ///
    /// This is used to allocate buffers that strings will be written to.  The value is generally
    /// the number of codepoints * 3, which is pessimistic but easy to calculate.
    StrMinByteLen {
        string: Box<Expression>,
    },
    /// Concatinate a list of values into a single string
    ///
    /// Each value can be a string, integer, or float value
    StrConcat {
        values: Vec<Expression>,
    },
    /// Create a new list
    ListCreate {
        inner: Type,
    },
    /// Get the number of elements in a list
    ListLen {
        list: Box<Expression>,
    },
    /// Get a value from the middle of the list
    ListGet {
        list: Box<Expression>,
        index: Box<Expression>,
    },
    /// Pop the value from the end of a list.  The list varable must be declared mutable.  Throws
    /// an exception if there are no elements in the list.
    ListPop {
        list: Box<Expression>,
    },
    /// Create a new map
    MapCreate {
        key: Type,
        value: Type,
    },
    /// Get the number of elements in a map
    MapLen {
        map: Box<Expression>,
    },
    /// Get a value a map.  Returns a nullable value.
    MapGet {
        map: Box<Expression>,
        key: Box<Expression>,
    },
    LiteralBoolean {
        value: bool,
    },
    LiteralString {
        value: String,
    },
    LiteralInt {
        value: String,
    },
    LiteralInt16 {
        value: String,
    },
    LiteralInt32 {
        value: String,
    },
    LiteralInt64 {
        value: String,
    },
    LiteralInt8 {
        value: String,
    },
    LiteralUInt16 {
        value: String,
    },
    LiteralUInt32 {
        value: String,
    },
    LiteralUInt64 {
        value: String,
    },
    LiteralUInt8 {
        value: String,
    },
    LiteralFloat32 {
        value: String,
    },
    LiteralFloat64 {
        value: String,
    },
    LiteralNull,
    /// Create a new BufferStream, consuming the pointer.
    BufStreamCreate {
        // Must match the name field in BufferStreamDef
        name: ClassName,
        pointer: Box<Expression>,
        size: Box<Expression>,
    },
    /// Consume a BufferStream instance and return the raw pointer
    BufStreamIntoPointer {
        // Must match the name field in BufferStreamDef
        name: ClassName,
        buf: Box<Expression>,
    },
    /// Get the current position of a buffer stream
    BufStreamPos {
        buf: Box<Expression>,
    },
    /// Get the size of a buffer stream
    BufStreamSize {
        buf: Box<Expression>,
    },
    // Read a u8 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadUInt8 {
        buf: Box<Expression>,
    },
    // Read a u16 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadUInt16 {
        buf: Box<Expression>,
    },
    // Read a u32 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadUInt32 {
        buf: Box<Expression>,
    },
    // Read a u64 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadUInt64 {
        buf: Box<Expression>,
    },
    // Read a i8 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadInt8 {
        buf: Box<Expression>,
    },
    // Read a i16 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadInt16 {
        buf: Box<Expression>,
    },
    // Read a i32 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadInt32 {
        buf: Box<Expression>,
    },
    // Read a i64 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadInt64 {
        buf: Box<Expression>,
    },
    // Read a f32 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadFloat32 {
        buf: Box<Expression>,
    },
    // Read a f64 value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadFloat64 {
        buf: Box<Expression>,
    },
    // Read a string value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadString {
        buf: Box<Expression>,
        /// Number of bytes to read
        size: Box<Expression>,
    },
    // Read a pointer value from a buffer stream.  The variable must be declared mutable.
    BufStreamReadPointer {
        name: String,
        buf: Box<Expression>,
    },
}

impl Expression {
    pub fn into_statement(self) -> Statement {
        Statement::Expression { expr: self }
    }
}
