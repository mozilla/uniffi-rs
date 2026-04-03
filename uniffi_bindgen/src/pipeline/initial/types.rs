/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_type(ty: uniffi_meta::Type, context: &Context) -> Result<Type> {
    Ok(match ty {
        uniffi_meta::Type::UInt8 => Type::UInt8,
        uniffi_meta::Type::Int8 => Type::Int8,
        uniffi_meta::Type::UInt16 => Type::UInt16,
        uniffi_meta::Type::Int16 => Type::Int16,
        uniffi_meta::Type::UInt32 => Type::UInt32,
        uniffi_meta::Type::Int32 => Type::Int32,
        uniffi_meta::Type::UInt64 => Type::UInt64,
        uniffi_meta::Type::Int64 => Type::Int64,
        uniffi_meta::Type::Float32 => Type::Float32,
        uniffi_meta::Type::Float64 => Type::Float64,
        uniffi_meta::Type::Boolean => Type::Boolean,
        uniffi_meta::Type::String => Type::String,
        uniffi_meta::Type::Bytes => Type::Bytes,
        uniffi_meta::Type::Timestamp => Type::Timestamp,
        uniffi_meta::Type::Duration => Type::Duration,
        uniffi_meta::Type::Box { inner_type } => Type::Box {
            inner_type: inner_type.map_node(context)?,
        },
        uniffi_meta::Type::Optional { inner_type } => Type::Optional {
            inner_type: inner_type.map_node(context)?,
        },
        uniffi_meta::Type::Sequence { inner_type } => Type::Sequence {
            inner_type: inner_type.map_node(context)?,
        },
        uniffi_meta::Type::Map {
            key_type,
            value_type,
        } => Type::Map {
            key_type: key_type.map_node(context)?,
            value_type: value_type.map_node(context)?,
        },
        uniffi_meta::Type::Object {
            module_path,
            name,
            imp,
        } => Type::Interface {
            namespace: context.get_namespace_name(&module_path)?,
            orig_name: context.get_orig_name(&module_path, &name),
            name,
            imp: imp.map_node(context)?,
        },
        uniffi_meta::Type::Record { module_path, name } => Type::Record {
            namespace: context.get_namespace_name(&module_path)?,
            orig_name: context.get_orig_name(&module_path, &name),
            name,
        },
        uniffi_meta::Type::Enum { module_path, name } => Type::Enum {
            namespace: context.get_namespace_name(&module_path)?,
            orig_name: context.get_orig_name(&module_path, &name),
            name,
        },
        uniffi_meta::Type::CallbackInterface { module_path, name } => Type::CallbackInterface {
            namespace: context.get_namespace_name(&module_path)?,
            orig_name: context.get_orig_name(&module_path, &name),
            name,
        },
        uniffi_meta::Type::Custom {
            module_path,
            name,
            builtin,
        } => Type::Custom {
            namespace: context.get_namespace_name(&module_path)?,
            orig_name: context.get_orig_name(&module_path, &name),
            name,
            builtin: builtin.map_node(context)?,
        },
    })
}

pub fn map_record(input: uniffi_meta::RecordMetadata, context: &Context) -> Result<Record> {
    Ok(Record {
        constructors: context.constructors_for_type(&input.module_path, &input.name)?,
        methods: context.methods_for_type(&input.module_path, &input.name)?,
        uniffi_traits: context.uniffi_traits_for_type(&input.module_path, &input.name)?,
        orig_name: input.orig_name.unwrap_or_else(|| input.name.clone()),
        name: input.name,
        module_path: input.module_path,
        fields: input.fields.map_node(context)?,
        docstring: input.docstring,
    })
}

pub fn map_enum(input: uniffi_meta::EnumMetadata, context: &Context) -> Result<Enum> {
    Ok(Enum {
        constructors: context.constructors_for_type(&input.module_path, &input.name)?,
        methods: context.methods_for_type(&input.module_path, &input.name)?,
        uniffi_traits: context.uniffi_traits_for_type(&input.module_path, &input.name)?,
        orig_name: input.orig_name.unwrap_or_else(|| input.name.clone()),
        name: input.name,
        module_path: input.module_path,
        shape: input.shape,
        variants: input.variants.map_node(context)?,
        discr_type: input.discr_type.map_node(context)?,
        docstring: input.docstring,
    })
}

pub fn map_interface(input: uniffi_meta::ObjectMetadata, context: &Context) -> Result<Interface> {
    Ok(Interface {
        constructors: context.constructors_for_type(&input.module_path, &input.name)?,
        methods: context.methods_for_type(&input.module_path, &input.name)?,
        uniffi_traits: context.uniffi_traits_for_type(&input.module_path, &input.name)?,
        trait_impls: context.trait_impls_for_type(&input.module_path, &input.name)?,
        orig_name: input.orig_name.unwrap_or_else(|| input.name.clone()),
        name: input.name,
        module_path: input.module_path,
        docstring: input.docstring,
        imp: input.imp,
    })
}

pub fn map_callback_interface(
    input: uniffi_meta::CallbackInterfaceMetadata,
    context: &Context,
) -> Result<CallbackInterface> {
    Ok(CallbackInterface {
        methods: context.methods_for_type(&input.module_path, &input.name)?,
        // Renaming callback interfaces is not supported yet -- just copy name.
        orig_name: input.name.clone(),
        name: input.name,
        module_path: input.module_path,
        docstring: input.docstring,
    })
}

pub fn map_custom_type(
    input: uniffi_meta::CustomTypeMetadata,
    context: &Context,
) -> Result<CustomType> {
    Ok(CustomType {
        orig_name: input.orig_name.unwrap_or(input.name.clone()),
        name: input.name,
        module_path: input.module_path,
        builtin: input.builtin.map_node(context)?,
        docstring: input.docstring,
    })
}

pub fn map_variant(input: uniffi_meta::VariantMetadata, context: &Context) -> Result<Variant> {
    Ok(Variant {
        orig_name: input.orig_name.unwrap_or_else(|| input.name.clone()),
        name: input.name,
        discr: input.discr.map_node(context)?,
        fields: input.fields.map_node(context)?,
        docstring: input.docstring,
    })
}

pub fn map_field(input: uniffi_meta::FieldMetadata, context: &Context) -> Result<Field> {
    Ok(Field {
        orig_name: input.orig_name.unwrap_or_else(|| input.name.clone()),
        name: input.name,
        ty: input.ty.map_node(context)?,
        default: input.default.map_node(context)?,
        docstring: input.docstring,
    })
}
