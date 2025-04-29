/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    module.visit_mut(|ty: &mut TypeNode| {
        let package_name = match &ty.ty {
            Type::Enum {
                external_package_name,
                ..
            }
            | Type::Record {
                external_package_name,
                ..
            }
            | Type::Interface {
                external_package_name,
                ..
            }
            | Type::CallbackInterface {
                external_package_name,
                ..
            }
            | Type::Custom {
                external_package_name,
                ..
            } => external_package_name.as_ref(),
            _ => None,
        };
        if let Some(package_name) = package_name {
            ty.ffi_converter_name =
                format!("{package_name}._UniffiFfiConverter{}", ty.canonical_name);
        } else {
            ty.ffi_converter_name = format!("_UniffiFfiConverter{}", ty.canonical_name);
        }
        ty.type_name = type_name(&ty.ty);
    });
    module.visit_mut(|return_ty: &mut ReturnType| {
        return_ty.type_name = match &return_ty.ty {
            Some(type_node) => type_node.type_name.clone(),
            None => "None".to_string(),
        }
    });
    Ok(())
}

fn type_name(ty: &Type) -> String {
    match ty {
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
            external_package_name,
            name,
            ..
        }
        | Type::Record {
            external_package_name,
            name,
            ..
        }
        | Type::Enum {
            external_package_name,
            name,
            ..
        }
        | Type::CallbackInterface {
            external_package_name,
            name,
            ..
        }
        | Type::Custom {
            external_package_name,
            name,
            ..
        } => match external_package_name {
            None => name.clone(),
            Some(package) => format!("{package}.{name}"),
        },
        Type::Optional { inner_type } => {
            format!("typing.Optional[{}]", type_name(inner_type))
        }
        Type::Sequence { inner_type } => {
            format!("typing.List[{}]", type_name(inner_type))
        }
        Type::Map {
            key_type,
            value_type,
        } => format!("dict[{}, {}]", type_name(key_type), type_name(value_type)),
    }
}
