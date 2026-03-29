/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_function(input: general::Function, context: &Context) -> Result<Function> {
    let module_path = context.normalize_rust_module_path(&input.module_path)?;
    let fully_qualified_name_rs = format!(
        "{module_path}::{}",
        names::escape_rust(&input.callable.orig_name)
    );
    Ok(Function {
        docstring: input.docstring,
        jni_method_name: format!(
            "function{}{}",
            context.current_crate_name()?.to_upper_camel_case(),
            input.callable.name.to_upper_camel_case()
        ),
        callable: map_callable(input.callable, fully_qualified_name_rs, context)?,
    })
}

fn map_callable(
    input: general::Callable,
    fully_qualified_name_rs: String,
    context: &Context,
) -> Result<Callable> {
    let return_type = input.return_type.ty.map_node(context)?;
    let throws_type = input.throws_type.ty.map_node(context)?;
    let return_ffi = match &return_type {
        Some(type_node) if type_node.ffi_types.len() == 1 => ReturnFfi::Primitive {
            type_node: type_node.clone(),
            ffi_type: type_node.ffi_types[0],
        },
        Some(_type_node) => {
            todo!("returns with more than one ffi type")
        }
        None => ReturnFfi::Void,
    };

    Ok(Callable {
        fully_qualified_name_rs,
        kind: input.kind.map_node(context)?,
        is_async: input.async_data.is_some(),
        name: input.name,
        arguments: map_arguments(input.arguments, context)?,
        return_type,
        throws_type,
        return_ffi,
    })
}

fn map_arguments(input: Vec<general::Argument>, context: &Context) -> Result<Vec<Argument>> {
    let mut allocator = FfiArgAllocator::default();
    input
        .into_iter()
        .map(|arg| {
            let ty = arg.ty.map_node(context)?;
            let ffi_args = allocator.create_ffi_args(&ty);
            Ok(Argument {
                name: arg.name,
                optional: arg.optional,
                ty,
                ffi_args,
            })
        })
        .collect()
}

impl Callable {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn ffi_arguments(&self) -> impl Iterator<Item = &FfiArgument> {
        self.arguments.iter().flat_map(|a| &a.ffi_args)
    }

    pub fn return_type_kt(&self) -> &str {
        match &self.return_type {
            None => "Unit",
            Some(ty) => &ty.type_kt,
        }
    }

    pub fn arg_list_kt(&self) -> String {
        self.arguments
            .iter()
            .map(|a| format!("{}: {}", a.name_kt(), a.ty.type_kt))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Argument {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}

/// Generates argument names for FFI arguments that we're passing
#[derive(Default)]
struct FfiArgAllocator(usize);

impl FfiArgAllocator {
    fn create_ffi_args(&mut self, ty: &TypeNode) -> Vec<FfiArgument> {
        ty.ffi_types
            .iter()
            .map(|ffi_type| FfiArgument {
                name: self.next(),
                ty: *ffi_type,
            })
            .collect()
    }

    fn next(&mut self) -> String {
        let i = self.0;
        self.0 += 1;
        format!("uniffi_arg_{i}")
    }
}
