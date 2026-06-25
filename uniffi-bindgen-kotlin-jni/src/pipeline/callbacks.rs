/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_callback_interface(
    input: general::CallbackInterface,
    context: &Context,
) -> Result<CallbackInterface> {
    let self_type = input.self_type.map_node(context)?;
    Ok(CallbackInterface {
        methods: map_methods(&self_type, input.vtable, context)?,
        module_path: context.rust_module_path_for_type(&self_type.ty)?,
        self_type,
        name: input.name,
        orig_name: input.orig_name,
        docstring: input.docstring,
        crate_name: context.current_crate_name()?.to_string(),
        for_trait_interface: false,
    })
}

pub fn map_trait_interface(
    input: general::Interface,
    context: &Context,
) -> Result<CallbackInterface> {
    let self_type = input.self_type.map_node(context)?;
    Ok(CallbackInterface {
        methods: map_methods(
            &self_type,
            input.vtable.ok_or_else(|| {
                anyhow!(
                    "UniFFI internal error in map_trait_interface: vtable is None ({})",
                    input.name
                )
            })?,
            context,
        )?,
        self_type,
        name: input.name,
        orig_name: input.orig_name,
        module_path: context.normalize_rust_module_path(&input.module_path)?,
        docstring: input.docstring,
        crate_name: context.current_crate_name()?.to_string(),
        for_trait_interface: true,
    })
}

fn map_methods(
    self_type: &TypeNode,
    vtable: general::VTable,
    context: &Context,
) -> Result<Vec<CallbackMethod>> {
    vtable
        .methods
        .into_iter()
        .enumerate()
        .map(|(method_index, m)| {
            // 1 million methods ought to be enough for everybody.
            let method_id = self_type.id * 1_000_000 + method_index as u64;
            let Some(self_name) = self_type.ty.name() else {
                bail!("Invalid Callable self type: {self_type:?}")
            };
            let module_path = context.rust_module_path_for_type(&self_type.ty)?;
            let fully_qualified_name_rs = format!(
                "{module_path}::{}::{}",
                names::escape_rust(self_name),
                names::escape_rust(&m.callable.orig_name)
            );
            Ok(CallbackMethod {
                dispatch_fn_rs: format!("callback_interface_dispatch_{method_id}"),
                dispatch_fn_kt: format!("callbackInterfaceDispatch{method_id}"),
                callable: callables::map_callable(m.callable, fully_qualified_name_rs, context)?,
            })
        })
        .collect()
}

pub fn interface_for_callback_interface(
    cbi: &general::CallbackInterface,
    context: &Context,
) -> Result<Interface> {
    Ok(Interface {
        name: cbi.name.to_upper_camel_case(),
        methods: cbi.methods.clone().map_node(context)?,
        docstring: cbi.docstring.clone(),
    })
}

impl CallbackInterface {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.orig_name)
    }

    pub fn has_async_method(&self) -> bool {
        self.methods.iter().any(|m| m.callable.is_async)
    }

    pub fn free_fn_kt(&self) -> String {
        format!("callbackInterfaceFree{}", self.self_type.id)
    }

    pub fn handle_map_kt(&self) -> String {
        format!("callbackInterfaceHandleMap{}", self.self_type.id)
    }

    pub fn impl_struct_rs(&self) -> String {
        format!("UniffiCallbackImpl{}", self.self_type.id)
    }
}

impl CallbackMethod {
    pub fn jni_signature(&self) -> String {
        let mut args = String::new();
        args.push('J'); // callback handle
        if self.has_return_pointer() {
            args.push('J');
        }
        if self.callable.is_async {
            args.push('J'); // oneshot handle
        }
        args.extend(
            self.callable
                .ffi_arguments()
                .map(|ffi_arg| ffi_arg.ty.jni_signature()),
        );
        let ret = if !self.callable.is_async {
            match self.callable.return_ffi() {
                ReturnFfi::Primitive { ffi_type, .. } => ffi_type.jni_signature(),
                _ => "V",
            }
        } else {
            "V"
        };

        format!("({args}){ret}")
    }

    // Should this method input a return pointer
    //
    // The return pointer is used to return non-primitive values and/or to return err values
    pub fn has_return_pointer(&self) -> bool {
        if self.callable.is_async {
            false
        } else {
            self.callable
                .return_type()
                .is_some_and(|return_type| !return_type.lowers_to_primitive())
                || self.callable.throws_type().is_some()
        }
    }

    pub fn jni_method_call_name(&self) -> &'static str {
        match self.callable.return_type() {
            Some(t) => match t.ffi_types.as_slice() {
                [FfiType::Int8] | [FfiType::Boolean] => "call_byte",
                [FfiType::Int16] => "call_short",
                [FfiType::Int32] => "call_int",
                [FfiType::Int64] => "call_long",
                [FfiType::Float32] => "call_float",
                [FfiType::Float64] => "call_double",
                [FfiType::String]
                | [FfiType::ByteBuffer]
                | [FfiType::ByteArray]
                | [FfiType::ShortArray]
                | [FfiType::IntArray]
                | [FfiType::LongArray]
                | [FfiType::FloatArray]
                | [FfiType::DoubleArray] => "call_object",
                // Return is handled through the return pointer
                _ => "call_void",
            },
            None => "call_void",
        }
    }

    pub fn default_return_kt(&self) -> &'static str {
        match self.callable.return_ffi() {
            ReturnFfi::Primitive { ffi_type, .. } => ffi_type.default_kt(),
            _ => "Unit",
        }
    }
}
