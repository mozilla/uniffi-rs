/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::rust_buffer;
use super::*;

pub fn map_namespace(namespace: initial::Namespace, context: &Context) -> Result<Namespace> {
    let mut child_context = context.clone();
    let context = &mut child_context;

    context.update_from_namespace(&namespace)?;

    Ok(Namespace {
        ffi_definitions: IndexSet::from_iter(sort::sort_ffi_definitions(
            [
                rust_buffer::ffi_definitions(context)?,
                checksums::ffi_definitions(&namespace)?,
                ffi_functions::ffi_definitions(&namespace, context)?,
                objects::ffi_definitions(&namespace, context)?,
                callback_interfaces::ffi_definitions(&namespace, context)?,
                rust_future::ffi_definitions(&namespace)?,
            ]
            .into_iter()
            .flatten(),
        )),
        checksums: checksums::checksums(&namespace)?,
        type_definitions: sort::sort_type_definitions(
            [
                // Map existing type definitions from the initial IR (record/enum/interface
                // definitions, etc).
                namespace.type_definitions.clone().map_node(context)?,
                // Add new type definitions for types found by walking the IR and finding all types
                // used.  This adds type definitions for:
                // * Simple types used in function signatures (`u8`, `bool`, etc).
                // * Compound types (`Vec<MyRecord>`).
                // * External types
                type_definitions_from_api::type_definitions(&namespace, context)?,
            ]
            .into_iter()
            .flatten(),
        ),
        ffi_rustbuffer_alloc: rust_buffer::rustbuffer_alloc_fn_name(context)?,
        ffi_rustbuffer_from_bytes: rust_buffer::rustbuffer_from_bytes_fn_name(context)?,
        ffi_rustbuffer_free: rust_buffer::rustbuffer_free_fn_name(context)?,
        ffi_rustbuffer_reserve: rust_buffer::rustbuffer_reserve_fn_name(context)?,
        ffi_uniffi_contract_version: checksums::ffi_uniffi_contract_version(&namespace),
        correct_contract_version: uniffi_meta::UNIFFI_CONTRACT_VERSION.to_string(),
        string_type_node: Type::String.map_node(context)?,
        name: namespace.name,
        crate_name: namespace.crate_name,
        config_toml: namespace.config_toml,
        docstring: namespace.docstring,
        functions: namespace.functions.map_node(context)?,
    })
}
