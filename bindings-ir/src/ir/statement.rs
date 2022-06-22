/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{names::*, Expression, Type};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum Statement {
    /// Declare a non-reassignable variable
    Val {
        /// Name of the variable
        name: VarName,
        /// Type of the variable
        #[serde(rename = "type")]
        type_: Type,
        /// Must match [Type]
        initial: Expression,
        /// Can the data inside this variable be changed?
        mutable: bool,
    },
    /// Declare a reassignable varable
    Var {
        /// Name of the variable
        name: VarName,
        /// Type of the variable
        #[serde(rename = "type")]
        type_: Type,
        /// Must match [Type]
        initial: Expression,
        /// Can the data inside this variable be changed?
        mutable: bool,
    },
    /// Assign a new value to a variable.  This is only allowed for primitive types that are
    /// declared mutable.
    Assign {
        name: VarName,
        value: Expression,
    },
    /// Set a value in a CStruct, Record, or Class instance.  This is only allowed for if both the
    /// variable storing the container and the field are declared mutable.
    Set {
        /// Owned CStruct, Record, or Class instance
        container: Expression,
        /// name of the field
        field: FieldName,
        /// Must match the type of [field]
        value: Expression,
    },
    /// Conditionally execute code based on a boolean value
    If {
        expr: Expression,
        then: Block,
        #[serde(rename = "else")]
        else_: Option<Block>,
    },
    /// Loop over the range [start, end), both must evaluate to ints.
    For {
        var: VarName,
        start: Expression,
        end: Expression,
        block: Block,
    },
    /// Contiuously loop until a break statement
    Loop {
        block: Block,
    },
    /// Break out of a For/Loop/ListIterate/MapIterate statement
    Break,
    /// Push a value to the end of a list.  The list variable must be declared mutable.
    ListPush {
        list: Expression,
        value: Expression,
    },
    /// Set a value from the middle of the list.  The list variable must be declared mutable.
    ListSet {
        list: Expression,
        index: Expression,
        value: Expression,
    },
    /// Remove all elements from a map.  The list variable must be declared mutable.
    ListEmpty {
        list: Expression,
    },
    /// Iterate through all elements in a list
    ListIterate {
        list: Expression,
        var: VarName,
        block: Block,
    },
    /// Push a value to the end of a list.  The map variable must be declared mutable.
    MapSet {
        map: Expression,
        key: Expression,
        value: Expression,
    },
    /// Remove one element. from a map.  The map variable must be declared mutable.
    MapRemove {
        map: Expression,
        key: Expression,
    },
    // Remove all elements from a map.  The map variable must be declared mutable.
    MapEmpty {
        map: Expression,
    },
    /// Iterate through all elements in a map
    MapIterate {
        map: Expression,
        key_var: VarName,
        val_var: VarName,
        block: Block,
    },
    /// Set the current position of a buffer stream
    BufStreamSetPos {
        buf: Expression,
        pos: Expression,
    },
    // Write a u8 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteUInt8 {
        buf: Expression,
        value: Expression,
    },
    // Write a u16 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteUInt16 {
        buf: Expression,
        value: Expression,
    },
    // Write a u32 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteUInt32 {
        buf: Expression,
        value: Expression,
    },
    // Write a u64 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteUInt64 {
        buf: Expression,
        value: Expression,
    },
    // Write a i8 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteInt8 {
        buf: Expression,
        value: Expression,
    },
    // Write a i16 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteInt16 {
        buf: Expression,
        value: Expression,
    },
    // Write a i32 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteInt32 {
        buf: Expression,
        value: Expression,
    },
    // Write a i64 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteInt64 {
        buf: Expression,
        value: Expression,
    },
    // Write a f32 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteFloat32 {
        buf: Expression,
        value: Expression,
    },
    // Write a f64 value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteFloat64 {
        buf: Expression,
        value: Expression,
    },
    // Write a string value to a buffer stream.  The variable must be declared mutable.
    BufStreamWriteString {
        buf: Expression,
        value: Expression,
    },
    // Write a pointer value to a buffer stream.  The variable must be declared mutable.
    BufStreamWritePointer {
        name: String,
        buf: Expression,
        value: Expression,
    },
    /// Conditionally execute code based on an enum variant
    MatchEnum {
        value: Expression,
        name: ClassName,
        /// Arms of of the match, each variant must be matched by an arm
        arms: Vec<MatchEnumArm>,
    },
    /// Conditionally execute code based on an int value
    MatchInt {
        #[serde(rename = "type")]
        type_: Type,
        value: Expression,
        /// Arms of of the match, each possible value must be matched by an arm
        arms: Vec<MatchIntArm>,
    },
    /// Conditionally execute code based on if a value is null or not
    MatchNullable {
        value: Expression,
        some_arm: MatchArmSome,
        null_arm: MatchArmNull,
    },
    /// Destructure a CStruct into it's individual fields.
    ///
    /// This allows you to get owned values for the fields.
    Destructure {
        // Name of the CStruct
        name: ClassName,
        cstruct: Expression,
        /// Variables to store each field, the length of this should match the number of fields
        vars: Vec<VarName>,
    },
    /// Return a value, only valid inside a function or method
    Return {
        value: Option<Expression>,
    },
    /// Raise an exception
    Raise {
        exception: Expression,
    },
    /// Raise a internal exception
    ///
    /// Internal exceptions are similar to Rust panics and should be used for bugs that can't be
    /// recovered from.  Internal exceptions can be thrown from any function and don't need to be
    /// declared in the `throws` field.
    RaiseInternalException {
        message: Expression,
    },
    /// Assert that an expression is true.  Only valid in test cases
    ///
    Assert {
        value: Expression,
    },
    /// AssertRaises that a statement raises an exception.  Only valid in test cases.
    AssertRaises {
        name: ClassName,
        stmt: Box<Statement>,
    },
    /// Test the AsString method of a raised exception.  Only valid in test cases
    AssertRaisesWithString {
        string_value: Expression,
        stmt: Box<Statement>,
    },
    /// For garbage collected languages, run a garbage collection pass
    Gc,
    /// Execute an expression
    #[serde(rename = "ExpressionStatement")]
    Expression {
        expr: Expression,
    },
}

/// A single arm of a MatchEnum statement.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum MatchEnumArm {
    /// Match a particular variant
    Variant {
        // Name of the Variant
        variant: ClassName,
        /// Variables for each variant field.  These will be available inside the code block
        vars: Vec<VarName>,
        /// Block of code to execute for this arm
        block: Block,
    },
    /// Match any other variants.  This must always come last in the list
    Default { block: Block },
}

/// A single arm of a MatchInt statement.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub enum MatchIntArm {
    /// Match a particular value
    Value {
        value: i32,
        /// Block of code to execute for this arm
        block: Block,
    },
    /// Match any other values.  This must always come last in the list
    Default { block: Block },
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct MatchArmSome {
    pub var: VarName,
    pub block: Block,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct MatchArmNull {
    pub block: Block,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
#[serde(tag = "ir_type")]
pub struct Block {
    pub statements: Vec<Statement>,
}
