/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Helper functions to create AST items
use super::*;

pub fn ident(name: impl Into<String>) -> Expression {
    let name = name.into();
    let mut components = name.split('.');
    // First component is the main variable
    let mut expr = Expression::Ident {
        name: VarName::new(components.next().unwrap()),
    };
    // Future components are fields
    for name in components {
        expr = get(expr, name);
    }
    expr
}

pub fn call(name: impl Into<String>, values: impl IntoIterator<Item = Expression>) -> Expression {
    Expression::Call {
        name: FunctionName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn method_call(
    obj: Expression,
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::MethodCall {
        obj: Box::new(obj),
        name: FunctionName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn static_method_call(
    class: impl Into<String>,
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::StaticMethodCall {
        class: ClassName::new(class.into()),
        name: FunctionName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn ffi_call(
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::FFICall {
        name: name.into(),
        values: values.into_iter().collect(),
    }
}

pub fn create_class(
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::ClassCreate {
        name: ClassName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn create_data_class(
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::DataClassCreate {
        name: ClassName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn create_cstruct(
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::CStructCreate {
        name: name.into(),
        values: values.into_iter().collect(),
    }
}

pub fn create_enum(
    name: impl Into<String>,
    variant: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::EnumCreate {
        name: ClassName::new(name.into()),
        variant: ClassName::new(variant.into()),
        values: values.into_iter().collect(),
    }
}

pub fn create_exception(
    name: impl Into<String>,
    values: impl IntoIterator<Item = Expression>,
) -> Expression {
    Expression::ExceptionCreate {
        name: ClassName::new(name.into()),
        values: values.into_iter().collect(),
    }
}

pub fn some(inner: Expression) -> Expression {
    Expression::Some {
        inner: Box::new(inner),
    }
}

pub fn unwrap(nullable: Expression, message: Expression) -> Expression {
    Expression::Unwrap {
        nullable: Box::new(nullable),
        message: Box::new(message),
    }
}

pub fn into_rust(obj: Expression) -> Expression {
    Expression::IntoRust { obj: Box::new(obj) }
}

pub fn get(container: Expression, field: impl Into<String>) -> Expression {
    Expression::Get {
        container: Box::new(container),
        field: FieldName::new(field.into()),
    }
}

pub fn this() -> Expression {
    Expression::This
}

pub fn add(type_: Type, first: Expression, second: Expression) -> Expression {
    Expression::Add {
        type_,
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn sub(type_: Type, first: Expression, second: Expression) -> Expression {
    Expression::Sub {
        type_,
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn div(type_: Type, first: Expression, second: Expression) -> Expression {
    Expression::Div {
        type_,
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn mul(type_: Type, first: Expression, second: Expression) -> Expression {
    Expression::Mul {
        type_,
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn eq(first: Expression, second: Expression) -> Expression {
    Expression::Eq {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn ne(first: Expression, second: Expression) -> Expression {
    not(eq(first, second))
}

pub fn gt(first: Expression, second: Expression) -> Expression {
    Expression::Gt {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn lt(first: Expression, second: Expression) -> Expression {
    Expression::Lt {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn ge(first: Expression, second: Expression) -> Expression {
    Expression::Ge {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn le(first: Expression, second: Expression) -> Expression {
    Expression::Le {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn and(first: Expression, second: Expression) -> Expression {
    Expression::And {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn or(first: Expression, second: Expression) -> Expression {
    Expression::Or {
        first: Box::new(first),
        second: Box::new(second),
    }
}

pub fn not(value: Expression) -> Expression {
    Expression::Not {
        value: Box::new(value),
    }
}

pub fn ref_(name: impl Into<String>) -> Expression {
    Expression::Ref {
        name: VarName::new(name.into()),
        mutable: false,
    }
}

pub fn ref_mut(name: impl Into<String>) -> Expression {
    Expression::Ref {
        name: VarName::new(name.into()),
        mutable: true,
    }
}

pub fn val(name: impl Into<String>, type_: Type, initial: Expression) -> Statement {
    Statement::Val {
        name: VarName::new(name.into()),
        type_,
        initial,
        mutable: false,
    }
}

pub fn mut_val(name: impl Into<String>, type_: Type, initial: Expression) -> Statement {
    Statement::Val {
        name: VarName::new(name.into()),
        type_,
        initial,
        mutable: true,
    }
}

pub fn var(name: impl Into<String>, type_: Type, initial: Expression) -> Statement {
    Statement::Var {
        name: VarName::new(name.into()),
        type_,
        initial,
        mutable: false,
    }
}

pub fn mut_var(name: impl Into<String>, type_: Type, initial: Expression) -> Statement {
    Statement::Var {
        name: VarName::new(name.into()),
        type_,
        initial,
        mutable: true,
    }
}

pub fn set(container: Expression, field: impl Into<String>, value: Expression) -> Statement {
    Statement::Set {
        container,
        field: FieldName::new(field.into()),
        value,
    }
}

pub fn destructure(
    name: impl Into<String>,
    cstruct: Expression,
    vars: impl IntoIterator<Item = &'static str>,
) -> Statement {
    Statement::Destructure {
        name: ClassName::new(name.into()),
        cstruct,
        vars: vars.into_iter().map(VarName::new).collect(),
    }
}

pub fn assign(name: impl Into<String>, value: Expression) -> Statement {
    Statement::Assign {
        name: VarName::new(name.into()),
        value,
    }
}

pub fn add_assign(type_: Type, name: impl Into<String>, amount: Expression) -> Statement {
    let name = name.into();
    assign(name.clone(), add(type_, ident(name), amount))
}

pub fn sub_assign(type_: Type, name: impl Into<String>, amount: Expression) -> Statement {
    let name = name.into();
    assign(name.clone(), sub(type_, ident(name), amount))
}

pub fn mul_assign(type_: Type, name: impl Into<String>, amount: Expression) -> Statement {
    let name = name.into();
    assign(name.clone(), mul(type_, ident(name), amount))
}

pub fn div_assign(type_: Type, name: impl Into<String>, amount: Expression) -> Statement {
    let name = name.into();
    assign(name.clone(), div(type_, ident(name), amount))
}

pub fn if_(expr: Expression, then: impl IntoIterator<Item = Statement>) -> Statement {
    Statement::If {
        expr,
        then: block(then),
        else_: None,
    }
}

pub fn if_else(
    expr: Expression,
    then: impl IntoIterator<Item = Statement>,
    else_: impl IntoIterator<Item = Statement>,
) -> Statement {
    Statement::If {
        expr,
        then: block(then),
        else_: Some(block(else_)),
    }
}

pub fn for_(
    var: impl Into<String>,
    start: Expression,
    end: Expression,
    statements: impl IntoIterator<Item = Statement>,
) -> Statement {
    Statement::For {
        var: VarName::new(var.into()),
        start,
        end,
        block: block(statements),
    }
}

pub fn loop_(statements: impl IntoIterator<Item = Statement>) -> Statement {
    Statement::Loop {
        block: block(statements),
    }
}

pub fn break_() -> Statement {
    Statement::Break
}

pub fn match_enum(
    value: Expression,
    name: impl Into<String>,
    arms: impl IntoIterator<Item = MatchEnumArm>,
) -> Statement {
    Statement::MatchEnum {
        value,
        name: ClassName::new(name.into()),
        arms: arms.into_iter().collect(),
    }
}

pub fn is_instance(value: Expression, base_class: impl Into<String>) -> Expression {
    Expression::IsInstance {
        value: Box::new(value),
        class: ClassName::new(base_class.into()),
    }
}

pub fn match_int(type_: Type, value: Expression, arms: impl IntoIterator<Item = MatchIntArm>) -> Statement {
    Statement::MatchInt {
        type_,
        value,
        arms: arms.into_iter().collect(),
    }
}

pub fn match_nullable(
    value: Expression,
    some_arm: MatchArmSome,
    null_arm: MatchArmNull,
) -> Statement {
    Statement::MatchNullable {
        value,
        some_arm,
        null_arm,
    }
}

pub fn raise(exception: Expression) -> Statement {
    Statement::Raise { exception }
}

pub fn raise_internal_exception(message: Expression) -> Statement {
    Statement::RaiseInternalException { message }
}

pub fn assert(value: Expression) -> Statement {
    Statement::Assert { value }
}

pub fn assert_eq(first: Expression, second: Expression) -> Statement {
    assert(eq(first, second))
}

pub fn assert_ne(first: Expression, second: Expression) -> Statement {
    assert(ne(first, second))
}

pub fn assert_raises(name: impl Into<String>, stmt: Statement) -> Statement {
    Statement::AssertRaises {
        name: ClassName::new(name.into()),
        stmt: Box::new(stmt),
    }
}

pub fn assert_raises_with_string(string_value: Expression, stmt: Statement) -> Statement {
    Statement::AssertRaisesWithString {
        string_value,
        stmt: Box::new(stmt),
    }
}

pub fn gc() -> Statement {
    Statement::Gc
}

pub fn return_(value: impl Into<Expression>) -> Statement {
    Statement::Return {
        value: Some(value.into()),
    }
}

pub fn return_void() -> Statement {
    Statement::Return { value: None }
}

pub fn empty_block() -> Block {
    Block { statements: vec![] }
}

pub fn block(statements: impl IntoIterator<Item = Statement>) -> Block {
    Block {
        statements: statements.into_iter().collect(),
    }
}

pub fn public() -> Visibility {
    Visibility::Public
}

pub fn private() -> Visibility {
    Visibility::Private
}

pub fn field(name: impl Into<String>, type_: Type) -> Field {
    Field {
        name: FieldName::new(name.into()),
        type_,
        mutable: false,
    }
}

pub fn mut_field(name: impl Into<String>, type_: Type) -> Field {
    Field {
        name: FieldName::new(name.into()),
        type_,
        mutable: true,
    }
}

pub fn arg(name: impl Into<String>, type_: Type) -> Argument {
    Argument {
        name: ArgName::new(name.into()),
        type_,
    }
}

pub fn exception_base_def(name: impl Into<String>) -> ExceptionBaseDef {
    ExceptionBaseDef::new(name.into())
}

pub fn exception_base_def_child(
    parent: impl Into<String>,
    name: impl Into<String>,
) -> ExceptionBaseDef {
    ExceptionBaseDef::new_child(parent, name)
}

pub fn uint8() -> Type {
    Type::UInt8
}

pub fn int8() -> Type {
    Type::Int8
}

pub fn uint16() -> Type {
    Type::UInt16
}

pub fn int16() -> Type {
    Type::Int16
}

pub fn uint32() -> Type {
    Type::UInt32
}

pub fn int32() -> Type {
    Type::Int32
}

pub fn uint64() -> Type {
    Type::UInt64
}

pub fn int64() -> Type {
    Type::Int64
}

pub fn float32() -> Type {
    Type::Float32
}

pub fn float64() -> Type {
    Type::Float64
}

pub fn pointer(name: impl Into<String>) -> Type {
    Type::Pointer { name: name.into() }
}

pub fn cstruct(name: impl Into<String>) -> Type {
    Type::CStruct {
        name: CStructName::new(name.into()),
    }
}

pub fn boolean() -> Type {
    Type::Boolean
}

pub fn string() -> Type {
    Type::String
}

pub fn object(class: impl Into<String>) -> Type {
    Type::Object {
        class: ClassName::new(class.into()),
    }
}

pub fn list(type_: Type) -> Type {
    Type::List {
        inner: Box::new(type_),
    }
}

pub fn map(key: Type, value: Type) -> Type {
    Type::Map {
        key: Box::new(key),
        value: Box::new(value),
    }
}

pub fn nullable(type_: Type) -> Type {
    Type::Nullable {
        inner: Box::new(type_),
    }
}

pub fn reference(type_: Type) -> Type {
    Type::Reference {
        inner: Box::new(type_),
        mutable: false,
    }
}

pub fn reference_mut(type_: Type) -> Type {
    Type::Reference {
        inner: Box::new(type_),
        mutable: true,
    }
}

pub fn pointer_size() -> Expression {
    Expression::PointerSize
}

pub mod cast {
    use super::*;

    macro_rules! cast_fn {
        ($name:ident, $kind:ident) => {
            pub fn $name(value: Expression) -> Expression {
                Expression::$kind {
                    value: Box::new(value),
                }
            }
        };
    }

    cast_fn!(int8, CastInt8);
    cast_fn!(int16, CastInt16);
    cast_fn!(int32, CastInt32);
    cast_fn!(int64, CastInt64);
    cast_fn!(uint8, CastUInt8);
    cast_fn!(uint16, CastUInt16);
    cast_fn!(uint32, CastUInt32);
    cast_fn!(uint64, CastUInt64);
}

pub mod string {
    use super::*;

    pub fn min_byte_len(string: Expression) -> Expression {
        Expression::StrMinByteLen {
            string: Box::new(string),
        }
    }

    pub fn concat(values: impl IntoIterator<Item = Expression>) -> Expression {
        Expression::StrConcat {
            values: values.into_iter().collect(),
        }
    }
}

pub mod list {
    use super::*;

    pub fn create(inner: Type) -> Expression {
        Expression::ListCreate { inner }
    }

    pub fn len(list: impl Into<String>) -> Expression {
        Expression::ListLen {
            list: Box::new(ident(list)),
        }
    }

    pub fn get(list: impl Into<String>, index: Expression) -> Expression {
        Expression::ListGet {
            list: Box::new(ident(list)),
            index: Box::new(index),
        }
    }

    pub fn pop(list: impl Into<String>) -> Expression {
        Expression::ListPop {
            list: Box::new(ident(list)),
        }
    }

    pub fn push(list: impl Into<String>, value: Expression) -> Statement {
        Statement::ListPush {
            list: ident(list),
            value,
        }
    }

    pub fn set(list: impl Into<String>, index: Expression, value: Expression) -> Statement {
        Statement::ListSet {
            list: ident(list),
            index,
            value,
        }
    }

    pub fn empty(list: impl Into<String>) -> Statement {
        Statement::ListEmpty { list: ident(list) }
    }

    pub fn iterate(
        list: impl Into<String>,
        var: impl Into<String>,
        statements: impl IntoIterator<Item = Statement>,
    ) -> Statement {
        Statement::ListIterate {
            list: ident(list),
            var: VarName::new(var.into()),
            block: block(statements),
        }
    }
}

pub mod map {
    use super::*;

    pub fn create(key: Type, value: Type) -> Expression {
        Expression::MapCreate { key, value }
    }

    pub fn len(map: impl Into<String>) -> Expression {
        Expression::MapLen {
            map: Box::new(ident(map)),
        }
    }

    pub fn get(map: impl Into<String>, key: Expression) -> Expression {
        Expression::MapGet {
            map: Box::new(ident(map)),
            key: Box::new(key),
        }
    }

    pub fn set(map: impl Into<String>, key: Expression, value: Expression) -> Statement {
        Statement::MapSet {
            map: ident(map),
            key,
            value,
        }
    }

    pub fn remove(map: impl Into<String>, key: Expression) -> Statement {
        Statement::MapRemove {
            map: ident(map),
            key,
        }
    }

    pub fn empty(map: impl Into<String>) -> Statement {
        Statement::MapEmpty { map: ident(map) }
    }

    pub fn iterate(
        map: impl Into<String>,
        key_var: impl Into<String>,
        val_var: impl Into<String>,
        statements: impl IntoIterator<Item = Statement>,
    ) -> Statement {
        Statement::MapIterate {
            map: ident(map),
            key_var: VarName::new(key_var.into()),
            val_var: VarName::new(val_var.into()),
            block: block(statements),
        }
    }
}

/// BufferStream helpers
pub mod buf {
    use super::*;

    pub fn create(name: impl Into<String>, pointer: Expression, size: Expression) -> Expression {
        Expression::BufStreamCreate {
            name: ClassName::new(name.into()),
            pointer: Box::new(pointer),
            size: Box::new(size),
        }
    }

    pub fn into_ptr(name: impl Into<String>, buf: Expression) -> Expression {
        Expression::BufStreamIntoPointer {
            name: ClassName::new(name.into()),
            buf: Box::new(buf),
        }
    }

    pub fn pos(buf: Expression) -> Expression {
        Expression::BufStreamPos { buf: Box::new(buf) }
    }

    pub fn set_pos(buf: Expression, pos: Expression) -> Statement {
        Statement::BufStreamSetPos { buf: buf, pos: pos }
    }

    pub fn size(buf: Expression) -> Expression {
        Expression::BufStreamSize { buf: Box::new(buf) }
    }

    pub fn read_string(buf: Expression, size: Expression) -> Expression {
        Expression::BufStreamReadString {
            buf: Box::new(buf),
            size: Box::new(size),
        }
    }

    pub fn write_string(buf: Expression, value: Expression) -> Statement {
        Statement::BufStreamWriteString {
            buf: buf,
            value: value,
        }
    }

    pub fn read_pointer(name: impl Into<String>, buf: Expression) -> Expression {
        Expression::BufStreamReadPointer {
            name: name.into(),
            buf: Box::new(buf),
        }
    }

    pub fn write_pointer(name: impl Into<String>, buf: Expression, value: Expression) -> Statement {
        Statement::BufStreamWritePointer {
            name: name.into(),
            buf: buf,
            value: value,
        }
    }

    macro_rules! read_write_number_fns {
        ($type:ident, $type_camel:ident) => {
            paste::paste! {
                pub fn [<read_ $type>](buf: Expression) -> Expression {
                    Expression::[<BufStreamRead $type_camel>] {
                        buf: Box::new(buf),
                    }
                }

                pub fn [<write_ $type>](buf: Expression, value: Expression) -> Statement {
                    Statement::[<BufStreamWrite $type_camel>] {
                        buf: buf,
                        value: value,
                    }
                }
            }
        };
    }
    read_write_number_fns!(uint8, UInt8);
    read_write_number_fns!(uint16, UInt16);
    read_write_number_fns!(uint32, UInt32);
    read_write_number_fns!(uint64, UInt64);
    read_write_number_fns!(int8, Int8);
    read_write_number_fns!(int16, Int16);
    read_write_number_fns!(int32, Int32);
    read_write_number_fns!(int64, Int64);
    read_write_number_fns!(float32, Float32);
    read_write_number_fns!(float64, Float64);
}

pub mod arm {
    use super::*;

    pub fn variant(
        variant: impl Into<String>,
        vars: impl IntoIterator<Item = String>,
        statements: impl IntoIterator<Item = Statement>,
    ) -> MatchEnumArm {
        MatchEnumArm::Variant {
            variant: ClassName::new(variant.into()),
            vars: vars.into_iter().map(VarName::new).collect(),
            block: block(statements),
        }
    }

    pub fn variant_default(statements: impl IntoIterator<Item = Statement>) -> MatchEnumArm {
        MatchEnumArm::Default {
            block: block(statements),
        }
    }

    pub fn int(value: i32, statements: impl IntoIterator<Item = Statement>) -> MatchIntArm {
        MatchIntArm::Value {
            value,
            block: block(statements),
        }
    }

    pub fn int_default(statements: impl IntoIterator<Item = Statement>) -> MatchIntArm {
        MatchIntArm::Default {
            block: block(statements),
        }
    }

    pub fn null(statements: impl IntoIterator<Item = Statement>) -> MatchArmNull {
        MatchArmNull {
            block: block(statements),
        }
    }

    pub fn some(
        var: impl Into<String>,
        statements: impl IntoIterator<Item = Statement>,
    ) -> MatchArmSome {
        MatchArmSome {
            var: VarName::new(var.into()),
            block: block(statements),
        }
    }
}

pub mod lit {
    use super::*;

    pub fn boolean(value: bool) -> Expression {
        Expression::LiteralBoolean { value }
    }

    pub fn int(value: i32) -> Expression {
        Expression::LiteralInt {
            value: value.to_string(),
        }
    }

    pub fn int8(value: i8) -> Expression {
        Expression::LiteralInt8 {
            value: value.to_string(),
        }
    }

    pub fn uint8(value: u8) -> Expression {
        Expression::LiteralUInt8 {
            value: value.to_string(),
        }
    }

    pub fn int16(value: i16) -> Expression {
        Expression::LiteralInt16 {
            value: value.to_string(),
        }
    }

    pub fn uint16(value: u16) -> Expression {
        Expression::LiteralUInt16 {
            value: value.to_string(),
        }
    }

    pub fn int32(value: i32) -> Expression {
        Expression::LiteralInt32 {
            value: value.to_string(),
        }
    }

    pub fn uint32(value: u32) -> Expression {
        Expression::LiteralUInt32 {
            value: value.to_string(),
        }
    }

    pub fn int64(value: i64) -> Expression {
        Expression::LiteralInt64 {
            value: value.to_string(),
        }
    }

    pub fn uint64(value: u64) -> Expression {
        Expression::LiteralUInt64 {
            value: value.to_string(),
        }
    }

    pub fn float32(value: &str) -> Expression {
        Expression::LiteralFloat32 {
            value: value.to_string(),
        }
    }

    pub fn float64(value: &str) -> Expression {
        Expression::LiteralFloat64 {
            value: value.to_string(),
        }
    }

    pub fn string(value: impl Into<String>) -> Expression {
        Expression::LiteralString {
            value: value.into(),
        }
    }

    pub fn null() -> Expression {
        Expression::LiteralNull
    }
}
