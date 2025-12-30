/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_type(mut ty: Type, _: &Context) -> Result<Type> {
    match &mut ty {
        Type::Enum { name, .. }
        | Type::Record { name, .. }
        | Type::Interface { name, .. }
        | Type::CallbackInterface { name, .. }
        | Type::Custom { name, .. } => {
            *name = names::type_name(name);
        }
        _ => (),
    }
    Ok(ty)
}

pub fn map_return_type(return_type: general::ReturnType, context: &Context) -> Result<ReturnType> {
    Ok(match return_type.ty {
        Some(ty) => {
            let ty = ty.map_node(context)?;
            ReturnType {
                type_name: ty.type_name.clone(),
                ty: Some(ty),
            }
        }
        None => ReturnType {
            ty: None,
            type_name: "None".to_string(),
        },
    })
}

pub fn type_name(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::Boolean => "bool".to_string(),
        Type::String => "str".to_string(),
        Type::Bytes => "bytes".to_string(),
        Type::Int8 => "int".to_string(),
        Type::Int16
        | Type::Int32
        | Type::Int64
        | Type::UInt8
        | Type::UInt16
        | Type::UInt32
        | Type::UInt64 => "int".to_string(),
        Type::Duration => "Duration".to_string(),
        Type::Timestamp => "Timestamp".to_string(),
        Type::Float32 | Type::Float64 => "float".to_string(),
        Type::Interface {
            namespace, name, ..
        }
        | Type::Record {
            namespace, name, ..
        }
        | Type::Enum {
            namespace, name, ..
        }
        | Type::CallbackInterface {
            namespace, name, ..
        }
        | Type::Custom {
            namespace, name, ..
        } => {
            let type_name = names::type_name(name);
            match context.external_package_name(namespace)? {
                None => type_name.clone(),
                Some(package) => format!("{package}.{type_name}"),
            }
        }
        Type::Optional { inner_type } => {
            format!("typing.Optional[{}]", type_name(inner_type, context)?)
        }
        Type::Sequence { inner_type } => {
            format!("typing.List[{}]", type_name(inner_type, context)?)
        }
        Type::Map {
            key_type,
            value_type,
        } => format!(
            "dict[{}, {}]",
            type_name(key_type, context)?,
            type_name(value_type, context)?
        ),
    })
}

pub fn ffi_converter_name(ty: &general::TypeNode, context: &Context) -> Result<String> {
    let ext_package = match ty.ty.namespace() {
        Some(namespace) => context.external_package_name(namespace)?,
        _ => None,
    };
    Ok(match ext_package {
        Some(package) => format!("{package}._UniffiFfiConverter{}", ty.canonical_name),
        None => format!("_UniffiFfiConverter{}", ty.canonical_name),
    })
}
